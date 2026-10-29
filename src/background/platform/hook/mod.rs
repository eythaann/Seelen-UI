//! Global hooks to listen to OS level events (windows of other apps, cursor, ...).

pub mod app_windows;
pub mod cursor;

use crate::error::Result;

pub fn register_hooks() -> Result<()> {
    log::trace!("Registering Windows and Virtual Desktop Hooks");
    app_windows::start();
    cursor::start();
    Ok(())
}
