use crate::{error::Result, windows_api::window::event::WinEvent, windows_api::window::Window};

use super::FancyToolbar;

impl FancyToolbar {
    pub fn process_win_event(&mut self, event: WinEvent, origin: &Window) -> Result<()> {
        if event == WinEvent::ObjectLocationChange && origin.hwnd() == self.hwnd()? {
            self.reposition_if_needed()?;
        }
        Ok(())
    }
}
