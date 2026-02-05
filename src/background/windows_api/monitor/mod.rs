mod brightness;

use windows::{
    Devices::Display::Core::{
        DisplayManager, DisplayTarget as WinRTDisplayTarget, DisplayView as WinRTDisplayView,
    },
    Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW, HMONITOR, MONITORINFOEXW},
};

use crate::{error::Result, windows_api::string_utils::WindowsString};
use seelen_core::{rect::Rect, system_state::MonitorId};

use super::{MonitorEnumerator, WindowsApi};

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

    pub fn info(&self) -> Result<MONITORINFOEXW> {
        WindowsApi::monitor_info(self.0)
    }

    pub fn devices(&self) -> Result<Vec<DisplayDevice>> {
        let info = self.info()?;
        let device = WindowsString::from_slice(&info.szDevice);

        let mut devices = Vec::new();
        unsafe {
            let mut idx = 0;
            loop {
                let mut display = DISPLAY_DEVICEW {
                    cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                    ..Default::default()
                };

                let success =
                    EnumDisplayDevicesW(device.as_pcwstr(), idx, &mut display, 1).as_bool();
                if !success {
                    break;
                }

                devices.push(display.into());
                idx += 1;
            }
        }

        Ok(devices)
    }

    pub fn get_primary_device(&self) -> Result<DisplayDevice> {
        let devices = self.devices()?;
        let device = devices.first().ok_or("no primary device")?;
        Ok(device.clone())
    }

    pub fn get_primary_target(&self) -> Result<MonitorTarget> {
        let device = self.get_primary_device()?;
        MonitorTarget::from_device_id(&device.id)
    }

    pub fn stable_id(&self) -> Result<String> {
        Ok(self.stable_id2()?.to_string())
    }

    pub fn stable_id2(&self) -> Result<MonitorId> {
        self.get_primary_target()?.stable_id()
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

/// Represents a win32 display device
#[derive(Clone)]
#[allow(dead_code)]
pub struct DisplayDevice {
    pub id: WindowsString,
    pub name: WindowsString,
    pub description: WindowsString,
    pub key: WindowsString,
    pub flags: u32,
}

impl From<DISPLAY_DEVICEW> for DisplayDevice {
    fn from(device: DISPLAY_DEVICEW) -> Self {
        Self {
            id: WindowsString::from_slice(&device.DeviceID),
            name: WindowsString::from_slice(&device.DeviceName),
            description: WindowsString::from_slice(&device.DeviceString),
            key: WindowsString::from_slice(&device.DeviceKey),
            flags: device.StateFlags.0,
        }
    }
}

impl std::fmt::Debug for DisplayDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DisplayDevice")
            .field("id", &self.id.to_os_string())
            .field("name", &self.name.to_os_string())
            .field("description", &self.description.to_os_string())
            .field("key", &self.key.to_os_string())
            .field("flags", &self.flags)
            .finish()
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
        let mut interfaces = Vec::new();
        for path in self.0.Paths()? {
            let target = path.Target()?;
            let interface_path = target.DeviceInterfacePath()?;
            interfaces.push(interface_path.to_string());
        }

        let win32_views = MonitorEnumerator::enumerate_win32()?;
        for win32_view in win32_views {
            let devices = win32_view.devices()?;
            for device in devices {
                if interfaces.contains(&device.id.to_string()) {
                    return Ok(win32_view);
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
    pub fn from_device_id(device_id: &WindowsString) -> Result<Self> {
        for target in DisplayManager::Create(Default::default())?.GetCurrentTargets()? {
            let Ok(id) = target.DeviceInterfacePath() else {
                continue;
            };

            if id == device_id.to_hstring() {
                return Ok(MonitorTarget(target));
            }
        }
        Err("Target for device id not found".into())
    }

    pub fn stable_id(&self) -> Result<MonitorId> {
        Ok(self.0.StableMonitorId()?.to_string().into())
    }

    pub fn name(&self) -> Result<String> {
        Ok(self.0.TryGetMonitor()?.DisplayName()?.to_string())
    }
}

impl From<WinRTDisplayTarget> for MonitorTarget {
    fn from(target: WinRTDisplayTarget) -> Self {
        Self(target)
    }
}
