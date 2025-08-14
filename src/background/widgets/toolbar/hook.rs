use crate::{
    error_handler::Result, windows_api::window::event::WinEvent, windows_api::window::Window,
};

use super::FancyToolbar;

impl FancyToolbar {
    pub fn process_win_event(&mut self, event: WinEvent, origin: &Window) -> Result<()> {
        match event {
            WinEvent::ObjectLocationChange => {
                if origin.hwnd() == self.hwnd()? {
                    self.set_position(origin.monitor().handle())?;
                }
            }
            WinEvent::SystemForeground | WinEvent::SyntheticForegroundLocationChange => {
                self.handle_overlaped_status(origin)?;
            }
            _ => {}
        };
        Ok(())
    }
}
