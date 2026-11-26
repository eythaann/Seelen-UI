#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use crate::{log_error, modules::app_management::AppManager};

use super::domain::{LifecycleEvent, LifecycleSystem};

#[cfg(windows)]
type Platform = windows::WindowsLifecycle;

pub struct LifecycleManager {
    platform: Platform,
}

impl LifecycleManager {
    /// Starts listening the OS lifecycle (first call only).
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<LifecycleManager> = LazyLock::new(|| {
            let mut m = LifecycleManager::create();
            log_error!(m.init());
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
        }
    }

    fn init(&mut self) -> crate::error::Result<()> {
        self.platform.init(Self::onplatform_event)
    }

    fn onplatform_event(event: LifecycleEvent) {
        match event {
            LifecycleEvent::Suspending => AppManager::instance().on_system_suspend(),
            LifecycleEvent::Resumed => AppManager::instance().on_system_resume(),
            LifecycleEvent::EndSession => {
                // Reacting here lets us exit promptly instead of idling in the exit channel
                // wait until the OS times out and force-kills us (reported as a hang).
                log::info!("Received shutdown/quiesce request, exiting service");
                crate::exit(0);
            }
        }
    }
}
