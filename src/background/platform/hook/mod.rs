//! Global hook on the windows of all the apps (created, destroyed, focused, moved, ...).

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::sync::atomic::AtomicBool;

use crate::event_manager;

use super::window::Window;

pub static LOG_WIN_EVENTS: AtomicBool = AtomicBool::new(false);

pub type AppWindowEvent = (WindowEvent, Window);

pub struct HookManager;

event_manager!(HookManager, AppWindowEvent);

/// Starts listening to the OS events, forwarding them through [`HookManager`].
pub fn start() {
    log::trace!("Registering Windows and Virtual Desktop Hooks");
    imp::start();
}

/// OS-agnostic events on the windows of other apps. Each OS translates its
/// native notifications into these; the ones nobody consumes are dropped there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowEvent {
    Created,
    Destroyed,
    Shown,
    Hidden,
    /// Stopped being hidden by the compositor (see [`Window::is_cloaked`]).
    Uncloaked,
    /// Became the foreground (active) window.
    Foregrounded,
    /// Received the keyboard focus.
    Focused,
    TitleChanged,
    ParentChanged,
    /// Some window state changed (enabled, visible, etc.), re-check it.
    StateChanged,
    Minimized,
    Unminimized,
    /// The user started moving or resizing the window.
    MoveSizeStarted,
    /// The user finished moving or resizing the window.
    MoveSizeEnded,
    /// Position or size changed, debounced per window.
    RectChanged,
    /// Position or size of the focused window changed, throttled.
    ForegroundRectChanged,
}
