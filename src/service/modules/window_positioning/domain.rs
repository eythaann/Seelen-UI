use seelen_core::{
    rect::Rect,
    system_state::{WindowHandle, WindowState},
};

use crate::error::Result;

/// OS contract of the window positioning module. Windows are identified by the native handle sent
/// by the app (`SvcAction`).
pub trait WindowPositioningSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// `asynchronous`: doesn't wait for the window to process the request (hung windows).
    fn set_state(&self, window: WindowHandle, state: WindowState, asynchronous: bool)
    -> Result<()>;

    /// Moves and resizes the window, without activating it nor changing its z-order.
    fn set_position(&self, window: WindowHandle, rect: &Rect) -> Result<()>;

    fn set_foreground(&self, window: WindowHandle) -> Result<()>;
}
