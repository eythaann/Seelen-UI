#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::{SysTrayIcon, SystrayIconAction};
use slu_ipc::messages::Win32TrayEvent;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

use super::domain::{SystemTrayEvent, SystemTraySystem};

#[cfg(windows)]
type Platform = windows::WindowsSystemTray;

/// Stateless: the icons are tied to their owner windows, so the native layer keeps them.
pub struct SystemTrayManager {
    platform: Platform,
}

event_manager!(SystemTrayManager, SystemTrayEvent);

impl SystemTrayManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<SystemTrayManager> = LazyLock::new(|| {
            let mut m = SystemTrayManager::create();
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

    /// Handles a tray event received via IPC.
    pub fn handle_tray_event(event: Win32TrayEvent) {
        Self::instance().platform.handle_hook_event(event);
    }

    /// Icons in the order the OS displays them, without the ones of dead apps.
    pub fn icons(&self) -> Vec<SysTrayIcon> {
        if self.platform.prune_dead_icons() {
            // keep every other tray consumer in sync, not only this caller
            Self::send(SystemTrayEvent::Changed);
        }
        self.platform.icons()
    }

    pub fn send_action(&self, session_id: &str, action: &SystrayIconAction) -> Result<()> {
        self.platform.send_action(session_id, action)
    }

    pub fn set_promoted(&self, session_id: &str, promoted: bool) -> Result<()> {
        self.platform.set_promoted(session_id, promoted)
    }

    pub fn set_order(&self, session_ids: &[String]) -> Result<()> {
        self.platform.set_order(session_ids)
    }
}
