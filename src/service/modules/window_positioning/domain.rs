use seelen_core::rect::Rect;

use crate::error::Result;

/// OS contract of the window positioning module.
///
/// Windows are identified by the raw native handle sent by the app (`SvcAction`), as well as the
/// show commands and position flags, which are passed through untouched.
pub trait WindowPositioningSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// `asynchronous`: doesn't wait for the window to process the request (hung windows).
    fn show_window(&self, window: isize, command: i32, asynchronous: bool) -> Result<()>;

    /// Moves and resizes the window, without activating it nor changing its z-order.
    fn set_position(&self, window: isize, rect: &Rect, flags: u32) -> Result<()>;

    fn set_foreground(&self, window: isize) -> Result<()>;
}
