//! Win32 `QueryDisplayConfig` helpers, used to correlate Win32 (`HMONITOR`) and WinRT
//! (`DisplayTarget`) display objects by matching adapter LUID + target ID against paths and
//! source mode positions.

use windows::{
    Devices::Display::Core::{DisplayManager, DisplayManagerOptions},
    Win32::{
        Devices::Display::{
            DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_DEVICE_INFO_HEADER,
            DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE, DISPLAYCONFIG_PATH_INFO,
            DISPLAYCONFIG_TARGET_DEVICE_NAME, DisplayConfigGetDeviceInfo,
            GetDisplayConfigBufferSizes, QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig,
        },
        Graphics::Gdi::HMONITOR,
    },
};

use seelen_core::system_state::MonitorId;

use crate::{error::Result, windows_api::WindowsApi};

/// Bridge struct holding the Win32 `QueryDisplayConfig` output.
pub struct DisplayConfigAndModes {
    pub paths: Vec<DISPLAYCONFIG_PATH_INFO>,
    pub modes: Vec<DISPLAYCONFIG_MODE_INFO>,
}

impl DisplayConfigAndModes {
    pub fn query_active() -> Result<Self> {
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

/// Returns the `DisplayConfigAndModes` snapshot along with the indices of the
/// `DISPLAYCONFIG_PATH_INFO` entries whose source mode sits at the monitor's
/// top-left corner on the virtual desktop (i.e. the paths that scan out to it).
pub fn matching_paths(hmonitor: HMONITOR) -> Result<(DisplayConfigAndModes, Vec<usize>)> {
    let info = WindowsApi::monitor_info(hmonitor)?;
    let rect = info.monitorInfo.rcMonitor;

    let display_config = DisplayConfigAndModes::query_active()?;
    let mut indices = Vec::new();
    for (idx, path) in display_config.paths.iter().enumerate() {
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
            indices.push(idx);
        }
    }
    Ok((display_config, indices))
}

/// Queries the target (monitor) names of a display path. On failure the names are left empty.
pub fn target_device_name(path: &DISPLAYCONFIG_PATH_INFO) -> DISPLAYCONFIG_TARGET_DEVICE_NAME {
    let mut target_name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            size: std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
            adapterId: path.targetInfo.adapterId,
            id: path.targetInfo.id,
        },
        ..Default::default()
    };
    unsafe {
        let _ = DisplayConfigGetDeviceInfo(&mut target_name.header);
    }
    target_name
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
pub fn winrt_stable_id_for_target(
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
