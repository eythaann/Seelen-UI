#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::MonitorId;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    system_modules::monitors::MonitorManager,
};

use super::domain::{BrightnessEvent, BrightnessSystem};

#[cfg(windows)]
type Platform = windows::WindowsBrightness;

/// Stateless: the panels are tied to their OS handles, so the native layer keeps them.
pub struct BrightnessManager {
    platform: Platform,
}

event_manager!(BrightnessManager, BrightnessEvent);

impl BrightnessManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<BrightnessManager> = LazyLock::new(|| {
            let mut m = BrightnessManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
        }
    }

    fn init(&mut self) -> Result<()> {
        // the panels belong to the monitors, so any topology change can add/remove them
        MonitorManager::subscribe(|_event| BrightnessManager::instance().platform.rescan());
        self.platform.init(Self::send)
    }

    /// Brightness of the monitor as a percentage, `None` if it can't be controlled.
    pub fn brightness(&self, id: &MonitorId) -> Option<u8> {
        self.platform.brightness(id)
    }

    /// `percent` goes from 0 to 100 and is mapped to whatever range each panel works with.
    pub fn set_brightness(&self, id: &MonitorId, percent: u8) -> Result<()> {
        self.platform.set_brightness(id, percent.min(100))
    }
}
