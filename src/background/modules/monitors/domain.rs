use seelen_core::system_state::PhysicalMonitor;

use crate::{
    error_handler::AppError,
    windows_api::{monitor::Monitor, WindowsApi},
};

impl TryFrom<Monitor> for PhysicalMonitor {
    type Error = AppError;
    fn try_from(m: Monitor) -> Result<Self, Self::Error> {
        let device = m.main_display_device()?;
        let dpi = WindowsApi::get_monitor_scale_factor(m.handle())?;
        Ok(Self {
            id: device.id(),
            name: device.description.to_string(),
            rect: m.rect()?,
            dpi,
        })
    }
}
