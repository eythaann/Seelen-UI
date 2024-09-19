use windows::Win32::Graphics::Gdi::HMONITOR;

use crate::error_handler::Result;

use super::WindowsApi;

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
}
