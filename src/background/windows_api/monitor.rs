use windows::Win32::Graphics::Gdi::HMONITOR;

use crate::{error_handler::Result, modules::input::domain::Point};

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

impl Monitor {
    pub fn id(&self) -> Result<String> {
        WindowsApi::monitor_name(self.0)
    }

    pub fn index(&self) -> Result<usize> {
        WindowsApi::monitor_index(self.0)
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
