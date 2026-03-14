mod brightness;

use windows::{
    Devices::Display::Core::{
        DisplayManager, DisplayManagerOptions, DisplayTarget as WinRTDisplayTarget,
        DisplayView as WinRTDisplayView,
    },
    Win32::{
        Devices::Display::{
            DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_DEVICE_INFO_HEADER,
            DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE, DISPLAYCONFIG_PATH_INFO,
            DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS,
        },
        Graphics::Gdi::{HMONITOR, MONITORINFOEXW},
    },
};

use crate::{error::Result, windows_api::string_utils::WindowsString};
use seelen_core::{rect::Rect, system_state::MonitorId};

use super::WindowsApi;

/// This struct represents a screen, a screen could be shown in multiple display devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monitor(HMONITOR);

unsafe impl Send for Monitor {}
unsafe impl Sync for Monitor {}

impl From<HMONITOR> for Monitor {
    fn from(hmonitor: HMONITOR) -> Self {
        Self(hmonitor)
    }
}

impl From<isize> for Monitor {
    fn from(hmonitor: isize) -> Self {
        Self(HMONITOR(hmonitor as _))
    }
}

impl From<&seelen_core::Point> for Monitor {
    fn from(point: &seelen_core::Point) -> Self {
        let hmonitor = WindowsApi::monitor_from_point(point);
        Self(hmonitor)
    }
}

// HMONITOR on win32 is the same concept as DisplayView in winrt

impl Monitor {
    pub fn handle(&self) -> HMONITOR {
        self.0
    }

    pub fn primary() -> Monitor {
        Monitor(WindowsApi::primary_monitor())
    }

    pub fn is_primary(&self) -> bool {
        self.0 == WindowsApi::primary_monitor()
    }

    pub fn at_point(point: &seelen_core::Point) -> Monitor {
        Monitor(WindowsApi::monitor_from_point(point))
    }

    pub fn info(&self) -> Result<MONITORINFOEXW> {
        WindowsApi::monitor_info(self.0)
    }

    /// Returns (MonitorId, friendly name) for this HMONITOR.
    ///
    /// Strategy:
    /// 1. Obtain the monitor's virtual-desktop position from `GetMonitorInfo`.
    /// 2. Find all `QueryDisplayConfig` paths whose source mode sits at that position.
    /// 3. For each candidate path try `winrt_stable_id_for_target`:
    ///    - WinRT `DisplayManager.GetCurrentTargets()` only surfaces physical display
    ///      targets. Virtual/render-only paths (e.g. the NVIDIA dGPU Optimus path) are
    ///      absent from that list, so they naturally produce no match and we continue
    ///      to the next candidate.
    ///    - The first path whose `targetInfo` matches a WinRT `DisplayTarget` gives us
    ///      the authoritative `StableMonitorId` and the friendly name.
    pub fn get_stable_info(&self) -> Result<(MonitorId, String)> {
        let info = WindowsApi::monitor_info(self.0)?;
        let rect = info.monitorInfo.rcMonitor;

        let display_config = DisplayConfigAndModes::query_active()?;
        for path in &display_config.paths {
            unsafe {
                // Only consider paths that have a source mode (desktop surface).
                let mode_idx = path.sourceInfo.Anonymous.modeInfoIdx as usize;
                if mode_idx == 0xFFFF_FFFF {
                    continue;
                }
                let Some(mode) = display_config.modes.get(mode_idx) else {
                    continue;
                };
                if mode.infoType != DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
                    continue;
                }
                // Match by top-left corner of the monitor on the virtual desktop.
                let pos = mode.Anonymous.sourceMode.position;
                if pos.x != rect.left || pos.y != rect.top {
                    continue;
                }

                // Query the DisplayConfig target device name for the friendly name.
                let mut target_name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
                    header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                        r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
                        size: std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
                        adapterId: path.targetInfo.adapterId,
                        id: path.targetInfo.id,
                    },
                    ..Default::default()
                };
                let _ = DisplayConfigGetDeviceInfo(&mut target_name.header);
                let friendly_name =
                    WindowsString::from_slice(&target_name.monitorFriendlyDeviceName).to_string();

                // Try WinRT lookup — virtual paths won't match and we'll skip them.
                if let Ok(result) = winrt_stable_id_for_target(
                    path.targetInfo.adapterId.LowPart,
                    path.targetInfo.adapterId.HighPart,
                    path.targetInfo.id,
                    friendly_name,
                ) {
                    return Ok(result);
                }
            }
        }
        Err("No WinRT DisplayTarget found for HMONITOR".into())
    }

    pub fn stable_id(&self) -> Result<MonitorId> {
        Ok(self.get_stable_info()?.0)
    }

    pub fn friendly_name(&self) -> Result<String> {
        Ok(self.get_stable_info()?.1)
    }

    pub fn rect(&self) -> Result<Rect> {
        let rect = WindowsApi::monitor_info(self.0)?.monitorInfo.rcMonitor;
        Ok(Rect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        })
    }

    pub fn scale_factor(&self) -> Result<f64> {
        let monitor_scale_factor = WindowsApi::get_monitor_scale_factor(self.0)?;
        let text_scale_factor = WindowsApi::get_text_scale_factor()?;
        Ok(monitor_scale_factor * text_scale_factor)
    }
}

// =================================================================================================
// =========================================== RT ==================================================
// =================================================================================================

/// represents a display screen view (one view can be shown in multiple displays 'mirrors')
///
/// # Safety
/// this is unsafe of store for long sessions as this object could be invalid on display changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayView(WinRTDisplayView);

impl DisplayView {
    pub fn as_win32_view(&self) -> Result<Monitor> {
        let display_config = DisplayConfigAndModes::query_active()?;

        for display_path in self.0.Paths()? {
            let target = display_path.Target()?;
            let adapter_id = target.Adapter()?.Id()?;
            let target_id = target.AdapterRelativeId()?;

            // Match directly by adapter LUID + target ID — no DeviceInterfacePath
            // string lookup or DisplayConfigGetDeviceInfo calls needed.
            for path in &display_config.paths {
                unsafe {
                    if path.targetInfo.adapterId.LowPart != adapter_id.LowPart
                        || path.targetInfo.adapterId.HighPart != adapter_id.HighPart
                        || path.targetInfo.id != target_id
                    {
                        continue;
                    }
                    let mode_idx = path.sourceInfo.Anonymous.modeInfoIdx as usize;
                    if mode_idx == 0xFFFF_FFFF {
                        continue;
                    }
                    let Some(mode) = display_config.modes.get(mode_idx) else {
                        continue;
                    };
                    if mode.infoType != DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE {
                        continue;
                    }

                    let pos = mode.Anonymous.sourceMode.position;
                    return Ok(Monitor::at_point(&seelen_core::Point {
                        x: pos.x,
                        y: pos.y,
                    }));
                }
            }
        }

        Err("Win32 Monitor not found for winrt view".into())
    }

    pub fn primary_target(&self) -> Result<MonitorTarget> {
        Ok(self.0.Paths()?.GetAt(0)?.Target()?.into())
    }
}

impl From<WinRTDisplayView> for DisplayView {
    fn from(view: WinRTDisplayView) -> Self {
        Self(view)
    }
}

/// represents a physical screen/monitor
pub struct MonitorTarget(WinRTDisplayTarget);

impl MonitorTarget {
    /// Returns the WinRT StableMonitorId — the canonical, authoritative stable ID.
    pub fn stable_id(&self) -> Result<MonitorId> {
        Ok(self.0.StableMonitorId()?.to_string().into())
    }
}

impl From<WinRTDisplayTarget> for MonitorTarget {
    fn from(target: WinRTDisplayTarget) -> Self {
        Self(target)
    }
}

// =================================================================================================
// ============================= DisplayConfig helpers (Win32-only) ================================
// =================================================================================================

/// Bridge struct holding the Win32 `QueryDisplayConfig` output.
///
/// Used to correlate Win32 (`HMONITOR`) and WinRT (`DisplayTarget`) display objects
/// by matching adapter LUID + target ID against paths and source mode positions.
struct DisplayConfigAndModes {
    paths: Vec<DISPLAYCONFIG_PATH_INFO>,
    modes: Vec<DISPLAYCONFIG_MODE_INFO>,
}

impl DisplayConfigAndModes {
    fn query_active() -> Result<Self> {
        unsafe {
            let mut num_paths: u32 = 0;
            let mut num_modes: u32 = 0;
            GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut num_paths, &mut num_modes)
                .ok()?;

            let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); num_paths as usize];
            let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); num_modes as usize];
            QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut num_paths,
                paths.as_mut_ptr(),
                &mut num_modes,
                modes.as_mut_ptr(),
                None,
            )
            .ok()?;
            paths.truncate(num_paths as usize);
            modes.truncate(num_modes as usize);
            Ok(Self { paths, modes })
        }
    }
}

/// Looks up the WinRT `DisplayTarget` matching the given adapter LUID and target ID,
/// and returns (StableMonitorId, friendly_name).
///
/// `DisplayManager.GetCurrentTargets()` only surfaces **physical** display targets.
/// Virtual or render-only paths (e.g. the NVIDIA dGPU Optimus indirect-display path)
/// are absent from the WinRT target list, so they naturally produce no match.
/// This means the caller does not need extra filtering — iterating over all QDC paths
/// at the monitor's position and calling this function for each one will succeed only
/// on the path that corresponds to the real scan-out adapter.
fn winrt_stable_id_for_target(
    luid_low: u32,
    luid_high: i32,
    target_id: u32,
    friendly_name: String,
) -> Result<(MonitorId, String)> {
    let dm = DisplayManager::Create(DisplayManagerOptions::None)?;
    for target in dm.GetCurrentTargets()? {
        let adapter_id = target.Adapter()?.Id()?;
        if adapter_id.LowPart != luid_low || adapter_id.HighPart != luid_high {
            continue;
        }
        if target.AdapterRelativeId()? != target_id {
            continue;
        }
        let stable_id: MonitorId = target.StableMonitorId()?.to_string().into();
        return Ok((stable_id, friendly_name));
    }
    Err("No WinRT DisplayTarget found for adapter/target".into())
}
