use getset::Getters;
use windows::Win32::{Foundation::RECT, Graphics::Gdi::HMONITOR};

use crate::{error_handler::Result, modules::input::domain::Point};

use super::{MonitorEnumerator, WindowsApi};

#[derive(Getters, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monitor {
    monitor: HMONITOR,
    #[getset(get = "pub")]
    work_area: RECT,
    #[getset(get = "pub")]
    device_pixel_ratio: u32,
}

unsafe impl Send for Monitor {}
unsafe impl Sync for Monitor {}

impl From<HMONITOR> for Monitor {
    fn from(hmonitor: HMONITOR) -> Self {
        Monitor::new(hmonitor)
    }
}

impl From<isize> for Monitor {
    fn from(hmonitor: isize) -> Self {
        Monitor::new(HMONITOR(hmonitor as _))
    }
}

impl From<&Point> for Monitor {
    fn from(point: &Point) -> Self {
        let hmonitor = WindowsApi::monitor_from_point(point);
        Monitor::new(hmonitor)
    }
}

impl Monitor {
    pub fn new(monitor: HMONITOR) -> Monitor {
        Monitor {
            monitor,
            work_area: WindowsApi::monitor_rect(monitor).ok().unwrap(),
            device_pixel_ratio: WindowsApi::get_device_pixel_ratio(monitor).ok().unwrap() as u32,
        }
    }

    pub fn update(&mut self) -> Result<()> {
        self.work_area = WindowsApi::monitor_rect(self.monitor)?;
        self.device_pixel_ratio = WindowsApi::get_device_pixel_ratio(self.monitor)? as u32;

        Ok(())
    }

    pub fn raw(&self) -> HMONITOR {
        self.monitor
    }

    pub fn id(&self) -> Result<String> {
        WindowsApi::monitor_name(self.monitor)
    }

    pub fn index(&self) -> Result<usize> {
        WindowsApi::monitor_index(self.monitor)
    }

    pub fn at(index: usize) -> Option<Monitor> {
        let monitors = MonitorEnumerator::get_all().ok()?;
        monitors.get(index).map(|m| Self::from(*m))
    }

    pub fn by_id(id: &str) -> Option<Monitor> {
        for m in MonitorEnumerator::get_all().ok()? {
            if let Ok(name) = WindowsApi::monitor_name(m) {
                if name == id {
                    return Some(Self::from(m));
                }
            }
        }
        None
    }
}
