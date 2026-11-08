pub mod cli;
pub mod handlers;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use crate::{error::Result, platform::window::Window};

/// Places the wallpaper widget window below the desktop icons, covering the whole virtual screen.
/// Falls back to a regular window at the bottom when the OS desktop layer can't be used.
fn attach_to_desktop(window: &Window) -> Result<()> {
    imp::attach_to_desktop(window)
}
