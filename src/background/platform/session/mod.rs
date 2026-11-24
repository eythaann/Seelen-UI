//! State of the OS user session the app runs in (not to be confused with `crate::session`,
//! which is the Seelen UI session).

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::sync::{
    LazyLock,
    atomic::{AtomicBool, Ordering},
};

use crate::error::Result;

/// Whether the session is interactive (not locked nor switched to another user).
static IS_INTERACTIVE: AtomicBool = AtomicBool::new(true);
/// Whether the session is locked (lock screen shown).
static IS_LOCKED: AtomicBool = AtomicBool::new(false);

/// Whether the session is interactive (not locked nor switched to another user).
/// Background threads and event processing should pause while it isn't.
#[inline]
pub fn is_interactive() -> bool {
    IS_INTERACTIVE.load(Ordering::Acquire)
}

/// Updates the interactive state, returning the previous one. Only called by the native layer
/// that listens to the OS session notifications.
#[inline]
pub fn set_interactive(interactive: bool) -> bool {
    IS_INTERACTIVE.swap(interactive, Ordering::AcqRel)
}

/// Whether the session is locked (lock screen shown). A locked session is never interactive.
#[inline]
pub fn is_locked() -> bool {
    IS_LOCKED.load(Ordering::Acquire)
}

/// Updates the locked state. Only called by the native layer that listens to the OS session
/// notifications.
#[inline]
pub fn set_locked(locked: bool) {
    IS_LOCKED.store(locked, Ordering::Release);
}

/// Identifier of the current OS logon session. Stable while the user stays logged in (app
/// restarts included) and different after a log off or a reboot, so it scopes the data that
/// must only survive app restarts.
pub fn logon_key() -> Result<String> {
    static KEY: LazyLock<std::result::Result<String, String>> =
        LazyLock::new(|| imp::logon_key().map_err(|e| e.to_string()));
    Ok(KEY.clone()?)
}
