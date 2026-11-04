mod display_config;
mod display_view;

use seelen_core::{Point, rect::Rect, system_state::MonitorId};
use windows::Win32::{
    Devices::Display::{
        DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO, DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE, DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO,
        DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE, DisplayConfigGetDeviceInfo,
        DisplayConfigSetDeviceInfo,
    },
    Foundation::RECT,
    Graphics::Gdi::HMONITOR,
};

use crate::{
    error::Result,
    modules::system_settings::application::SystemSettings,
    windows_api::{MonitorEnumerator, WindowsApi, string_utils::WindowsString},
};

use self::display_config::{matching_paths, target_device_name, winrt_stable_id_for_target};
use super::Monitor;

pub use self::display_view::DisplayView;

/// A screen of the virtual desktop, identified by its `HMONITOR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeMonitor(HMONITOR);

unsafe impl Send for NativeMonitor {}
unsafe impl Sync for NativeMonitor {}

impl NativeMonitor {
    pub fn primary() -> Self {
        Self(WindowsApi::primary_monitor())
    }

    pub fn at_point(point: &Point) -> Self {
        Self(WindowsApi::monitor_from_point(point))
    }

    pub fn enumerate() -> Result<Vec<Self>> {
        Ok(MonitorEnumerator::enumerate_win32()?
            .into_iter()
            .map(Self)
            .collect())
    }

    pub fn address(&self) -> isize {
        self.0.0 as isize
    }

    pub fn is_primary(&self) -> bool {
        self.0 == WindowsApi::primary_monitor()
    }

    pub fn rect(&self) -> Result<Rect> {
        Ok(from_win32_rect(
            WindowsApi::monitor_info(self.0)?.monitorInfo.rcMonitor,
        ))
    }

    pub fn work_area(&self) -> Result<Rect> {
        Ok(from_win32_rect(
            WindowsApi::monitor_info(self.0)?.monitorInfo.rcWork,
        ))
    }

    pub fn scale_factor(&self) -> Result<f64> {
        let monitor_scale_factor = WindowsApi::get_monitor_scale_factor(self.0)?;
        let text_scale_factor = SystemSettings::instance().get_text_scale_factor()?;
        Ok(monitor_scale_factor * text_scale_factor)
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
        let (display_config, indices) = matching_paths(self.0)?;
        for idx in indices {
            let path = &display_config.paths[idx];
            // Query the DisplayConfig target device name for the friendly name.
            let target_name = target_device_name(path);
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
        Err("No WinRT DisplayTarget found for HMONITOR".into())
    }

    /// Device interface paths of every target scanning out this monitor, e.g.
    /// `\\?\DISPLAY#BOE0900#4&10fd3ab1&0&UID265988#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}`.
    ///
    /// Unlike `get_stable_info` this doesn't stop at the first target WinRT knows about, so
    /// it also covers the other panels of a cloned group and the paths WinRT doesn't surface.
    fn target_device_paths(&self) -> Result<Vec<String>> {
        let (display_config, indices) = matching_paths(self.0)?;
        Ok(indices
            .into_iter()
            .map(|idx| {
                let target_name = target_device_name(&display_config.paths[idx]);
                WindowsString::from_slice(&target_name.monitorDevicePath).to_string()
            })
            .filter(|path| !path.is_empty())
            .collect())
    }

    /// Returns the HDR / advanced-color state of this monitor.
    /// `None` means the monitor/output does not support advanced color at all.
    pub fn hdr_state(&self) -> Result<Option<bool>> {
        let (display_config, indices) = matching_paths(self.0)?;
        for idx in indices {
            let path = &display_config.paths[idx];
            unsafe {
                let mut color_info = DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO {
                    header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                        r#type: DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO,
                        size: std::mem::size_of::<DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO>() as u32,
                        adapterId: path.targetInfo.adapterId,
                        id: path.targetInfo.id,
                    },
                    ..Default::default()
                };
                if DisplayConfigGetDeviceInfo(&mut color_info.header) != 0 {
                    continue;
                }
                let flags = color_info.Anonymous.value;
                let supported = flags & 0x1 != 0;
                let enabled = flags & 0x2 != 0;
                return Ok(supported.then_some(enabled));
            }
        }
        Ok(None)
    }

    /// Enables/disables HDR / advanced-color on this monitor.
    pub fn set_hdr_state(&self, enabled: bool) -> Result<()> {
        let (display_config, indices) = matching_paths(self.0)?;
        for idx in indices {
            let path = &display_config.paths[idx];
            unsafe {
                let mut state = DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE {
                    header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
                        r#type: DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE,
                        size: std::mem::size_of::<DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE>() as u32,
                        adapterId: path.targetInfo.adapterId,
                        id: path.targetInfo.id,
                    },
                    ..Default::default()
                };
                state.Anonymous.value = enabled as u32;
                let result = DisplayConfigSetDeviceInfo(&state.header);
                if result == 0 {
                    return Ok(());
                }
            }
        }
        Err("Failed to set HDR state: no matching display path found".into())
    }
}

impl From<isize> for NativeMonitor {
    fn from(address: isize) -> Self {
        Self(HMONITOR(address as _))
    }
}

/// Windows only concepts of a monitor, use it only from Windows specific code.
pub trait WindowsMonitorExt {
    fn handle(&self) -> HMONITOR;
    fn target_device_paths(&self) -> Result<Vec<String>>;
}

impl WindowsMonitorExt for Monitor {
    fn handle(&self) -> HMONITOR {
        self.0.0
    }

    fn target_device_paths(&self) -> Result<Vec<String>> {
        self.0.target_device_paths()
    }
}

impl From<HMONITOR> for Monitor {
    fn from(hmonitor: HMONITOR) -> Self {
        Self(NativeMonitor(hmonitor))
    }
}

fn from_win32_rect(rect: RECT) -> Rect {
    Rect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}
