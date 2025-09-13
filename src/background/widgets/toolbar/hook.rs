use crate::{error::Result, windows_api::window::event::WinEvent, windows_api::window::Window};

use super::FancyToolbar;

impl FancyToolbar {
    pub fn process_win_event(&mut self, event: WinEvent, origin: &Window) -> Result<()> {
        match event {
            WinEvent::SystemForeground | WinEvent::SyntheticForegroundLocationChange => {
                self.handle_overlaped_status(origin)?;
            }
            WinEvent::ObjectLocationChange => {
                if origin.hwnd() == self.hwnd()? {
                    self.reposition_if_needed()?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}
