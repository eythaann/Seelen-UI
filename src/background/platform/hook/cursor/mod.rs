//! Global cursor tracking.

#[cfg(windows)]
mod windows;

use seelen_core::Point;

use crate::error::Result;

#[cfg(windows)]
type Native = self::windows::WindowsCursorHook;

/// Contract to be implemented by each OS.
trait CursorHookApi {
    fn position() -> Result<Point>;
    /// Starts tracking the cursor, notifying the webviews about its movement.
    fn start();
}

/// Current position of the cursor on the virtual desktop.
pub fn position() -> Result<Point> {
    Native::position()
}

pub(super) fn start() {
    Native::start();
}
