#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

use super::domain::{FocusAssistEvent, FocusAssistSystem};

#[cfg(windows)]
type Platform = windows::WindowsFocusAssist;

pub struct FocusAssistManager {
    platform: Platform,
}

event_manager!(FocusAssistManager, FocusAssistEvent);

impl FocusAssistManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<FocusAssistManager> = LazyLock::new(|| {
            let mut m = FocusAssistManager::create();
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
        self.platform.init(Self::send)
    }

    pub fn is_active(&self) -> bool {
        self.platform.is_active().unwrap_or(false)
    }

    pub fn set_active(&self, enabled: bool) -> Result<()> {
        self.platform.set_active(enabled)
    }
}
