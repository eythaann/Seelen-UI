use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        EVENT_OBJECT_FOCUS, EVENT_OBJECT_NAMECHANGE, EVENT_SYSTEM_FOREGROUND,
    },
};

use crate::error_handler::Result;

use super::FancyToolbar;

impl FancyToolbar {
    pub fn process_win_event(&mut self, event: u32, hwnd: HWND) -> Result<()> {
        match event {
            EVENT_OBJECT_NAMECHANGE => {
                if self.last_focus == Some(hwnd.0) {
                    self.focus_changed(hwnd)?;
                }
            }
            EVENT_SYSTEM_FOREGROUND | EVENT_OBJECT_FOCUS => {
                self.focus_changed(hwnd)?;
            }
            _ => {}
        };
        Ok(())
    }
}
