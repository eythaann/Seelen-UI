use seelen_core::system_state::PhysicalMonitor;

use crate::{error::AppError, windows_api::monitor::Monitor};

impl TryFrom<Monitor> for PhysicalMonitor {
    type Error = AppError;
    fn try_from(m: Monitor) -> Result<Self, Self::Error> {
        Ok(Self {
            id: m.stable_id()?.into(),
            name: m.name()?,
            rect: m.rect()?,
            scale_factor: m.scale_factor()?,
            is_primary: m.is_primary(),
        })
    }
}
