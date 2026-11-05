use crate::{error::Result, windows_api::event_window::create_background_window};

/// The hidden window receiving the `WM_*` broadcasts (power, devices, session, ...).
pub fn start() -> Result<()> {
    create_background_window()
}
