//! Global cursor position.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use seelen_core::Point;

use crate::error::Result;

/// Current position of the cursor on the virtual desktop.
pub fn position() -> Result<Point> {
    imp::position()
}

/// Starts tracking the cursor, notifying the webviews about its movement.
pub fn start_tracking() {
    imp::start_tracking();
}
