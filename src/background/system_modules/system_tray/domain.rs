use seelen_core::system_state::{SysTrayIcon, SystrayIconAction};
use slu_ipc::messages::Win32TrayEvent;

use crate::error::Result;

/// OS contract of the system tray (notification area icons) module.
pub trait SystemTraySystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts tracking the icons, reporting changes through `on_event`.
    /// Called once; the tracking lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(SystemTrayEvent) + Send + Sync + 'static;

    /// Applies an icon change reported by the tray hook (through the service IPC).
    fn handle_hook_event(&self, event: Win32TrayEvent);

    /// Removes the icons whose owner died without removing them.
    /// Returns whether any was removed (not reported as an event).
    fn prune_dead_icons(&self) -> bool;

    /// Icons in the order the OS displays them.
    fn icons(&self) -> Vec<SysTrayIcon>;

    fn send_action(&self, session_id: &str, action: &SystrayIconAction) -> Result<()>;

    /// Shows the icon directly on the taskbar or only in the tray overflow.
    fn set_promoted(&self, session_id: &str, promoted: bool) -> Result<()>;

    /// Reorders the given icons, the others keep their position.
    fn set_order(&self, session_ids: &[String]) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum SystemTrayEvent {
    Changed,
}
