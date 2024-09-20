use windows::Win32::Graphics::Gdi::HMONITOR;

use crate::error_handler::Result;

use super::{MonitorEnumerator, WindowsApi};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monitor(HMONITOR);

impl From<HMONITOR> for Monitor {
    fn from(hmonitor: HMONITOR) -> Self {
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
