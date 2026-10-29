//! Events of the windows of all the apps (created, destroyed, focused, moved, ...).

#[cfg(windows)]
mod windows;

use std::sync::atomic::AtomicBool;

use crate::event_manager;

/// The event vocabulary is still the native one of each OS.
#[cfg(windows)]
pub use self::windows::AppWindowEvent;

#[cfg(windows)]
type Native = self::windows::WindowsAppWindowsHook;

pub static LOG_WIN_EVENTS: AtomicBool = AtomicBool::new(false);

pub struct HookManager;

event_manager!(HookManager, AppWindowEvent);

/// Contract to be implemented by each OS.
trait AppWindowsHookApi {
    /// Starts listening to the OS events, forwarding them through [`HookManager`].
    fn start();
}

pub(super) fn start() {
    Native::start();
}
