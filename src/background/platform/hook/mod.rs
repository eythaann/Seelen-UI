//! Global hook on the windows of all the apps (created, destroyed, focused, moved, ...).

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::sync::atomic::AtomicBool;

use crate::event_manager;

use super::window::Window;

/// The event vocabulary is still the native one of each OS.
#[cfg(windows)]
pub use self::windows::WinEvent;

pub type AppWindowEvent = (WinEvent, Window);

pub static LOG_WIN_EVENTS: AtomicBool = AtomicBool::new(false);

pub struct HookManager;

event_manager!(HookManager, AppWindowEvent);

/// Starts listening to the OS events, forwarding them through [`HookManager`].
pub fn start() {
    log::trace!("Registering Windows and Virtual Desktop Hooks");
    imp::start();
}
