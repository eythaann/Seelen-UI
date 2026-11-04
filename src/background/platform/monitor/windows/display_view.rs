//! WinRT display views and targets (`Windows.Devices.Display.Core`).

use windows::{
    Devices::Display::Core::{
        DisplayTarget as WinRTDisplayTarget, DisplayView as WinRTDisplayView,
    },
    Win32::{
        Devices::Display::DISPLAYCONFIG_MODE_INFO_TYPE_SOURCE,
        Foundation::HANDLE,
        Storage::FileSystem::{
            CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
            OPEN_EXISTING,
        },
    },
};
use windows_core::Owned;

use seelen_core::system_state::MonitorId;

use crate::{error::Result, platform::monitor::Monitor, windows_api::string_utils::WindowsString};

use super::display_config::DisplayConfigAndModes;

/// represents a display screen view (one view can be shown in multiple displays 'mirrors')
///
/// # Safety
/// this is unsafe of store for long sessions as this object could be invalid on display changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayView(WinRTDisplayView);

impl DisplayView {
    #[allow(dead_code)]
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

    /// Returns true if the primary target of this view reports a connected monitor.
    pub fn is_active(&self) -> windows_core::Result<bool> {
        self.0.Paths()?.GetAt(0)?.Target()?.IsConnected()
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

    /// Opens and returns a file handle for a display device using its DOS device path.\
    /// These handles are only used for the `DeviceIoControl` API (for internal displays);
    /// a handle can still be returned for external displays, but it should not be used.
    #[allow(dead_code)]
    fn get_file_handle(&self) -> Result<Owned<HANDLE>> {
        let device_id = self.0.TryGetMonitor()?.DeviceId()?.to_os_string();
        let device_id = WindowsString::from(device_id);

        // This could fail for virtual devices e.g. Remote Desktop sessions - they are not real monitors
        let handle = unsafe {
            CreateFileW(
                device_id.as_pcwstr(),
                (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                Default::default(),
                None,
            )?
        };
        Ok(unsafe { Owned::new(handle) })
    }
}

impl From<WinRTDisplayTarget> for MonitorTarget {
    fn from(target: WinRTDisplayTarget) -> Self {
        Self(target)
    }
}
