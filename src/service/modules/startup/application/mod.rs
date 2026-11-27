#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use crate::error::Result;

use super::domain::StartupSystem;

#[cfg(windows)]
type Platform = windows::WindowsStartup;

pub struct StartupManager {
    platform: Platform,
}

impl StartupManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<StartupManager> = LazyLock::new(StartupManager::create);
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
        }
    }

    /// this entry handles the startup of the service and the app on login
    pub fn register_service(&self) -> Result<()> {
        log::debug!("Creating service startup entry...");
        self.platform.register_service()
    }

    pub fn set_run_on_logon(&self, enabled: bool) -> Result<()> {
        self.platform.set_run_on_logon(enabled)
    }

    pub fn unregister_service(&self) -> Result<()> {
        self.platform.unregister_service()
    }
}
