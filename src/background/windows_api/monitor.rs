use windows::Win32::Graphics::Gdi::HMONITOR;
use windows_core::PCWSTR;

use crate::{error_handler::Result, modules::input::domain::Point};
use seelen_core::rect::Rect;

use super::{MonitorEnumerator, WindowsApi};

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

impl From<&Point> for Monitor {
    fn from(point: &Point) -> Self {
        let hmonitor = WindowsApi::monitor_from_point(point);
        Self(hmonitor)
    }
}

#[derive(Debug, Clone)]
pub struct DisplayDevice {
    pub id: String,
    pub name: String,
}

impl Monitor {
    pub fn handle(&self) -> HMONITOR {
        self.0
    }

    /// display device id
    pub fn device_id(&self) -> Result<String> {
        Ok(self.display_device()?.id)
    }

    pub fn is_primary(&self) -> Result<bool> {
        WindowsApi::monitor_get_is_primary(self.0)
    }

    pub fn display_device(&self) -> Result<DisplayDevice> {
        let device = WindowsApi::get_display_device(self.0)?;
        let buffer_id = device.DeviceID;
        let buffer_name = device.DeviceString;
        let id = PCWSTR::from_raw(buffer_id.as_ptr());
        let name = PCWSTR::from_raw(buffer_name.as_ptr());
        Ok(DisplayDevice {
            id: unsafe { id.to_string()? }
                .trim_start_matches(r"\\?\")
                .to_owned(),
            name: unsafe { name.to_string()? },
        })
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

    pub fn index(&self) -> Result<usize> {
        WindowsApi::monitor_index(self.0)
    }

    pub fn at(index: usize) -> Option<Monitor> {
        let monitors = MonitorEnumerator::get_all_v2().ok()?;
        monitors.get(index).copied()
    }

    pub fn by_id(id: &str) -> Option<Monitor> {
        for m in MonitorEnumerator::get_all_v2().ok()? {
            if let Ok(monitor_device_id) = m.device_id() {
                if monitor_device_id == id {
                    return Some(m);
                }
            }
        }
        None
    }
}
