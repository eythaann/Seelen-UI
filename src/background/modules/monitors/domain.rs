use seelen_core::system_state::PhysicalMonitor;

use crate::{
    error_handler::AppError,
    windows_api::{monitor::Monitor, WindowsApi},
};

impl TryFrom<Monitor> for PhysicalMonitor {
    type Error = AppError;
    fn try_from(m: Monitor) -> Result<Self, Self::Error> {
        let dpi = WindowsApi::get_monitor_scale_factor(m.handle())?;
        Ok(Self {
            id: m.stable_id()?,
            name: m.name()?,
            rect: m.rect()?,
            dpi,
        })
    }
}
