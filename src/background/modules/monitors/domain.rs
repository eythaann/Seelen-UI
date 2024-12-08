use serde::Serialize;

use crate::{
    error_handler::AppError,
    windows_api::{monitor::Monitor, WindowsApi},
};

#[derive(Debug, Clone, Serialize)]
pub struct PhysicalMonitor {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub dpi: f32,
}

impl TryFrom<Monitor> for PhysicalMonitor {
    type Error = AppError;
    fn try_from(m: Monitor) -> Result<Self, Self::Error> {
        let device = m.display_device()?;
        let rect = m.rect()?;
        let dpi = WindowsApi::get_device_pixel_ratio(m.raw())?;
        Ok(Self {
            id: device.id,
            name: device.name,
            width: (rect.right - rect.left) as u32,
            height: (rect.bottom - rect.top) as u32,
            dpi,
        })
    }
}
