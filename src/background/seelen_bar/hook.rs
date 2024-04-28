use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        EVENT_OBJECT_FOCUS, EVENT_OBJECT_NAMECHANGE, EVENT_SYSTEM_FOREGROUND,
    },
};

use crate::{error_handler::Result, seelen::SEELEN};

use super::FancyToolbar;

impl FancyToolbar {
    pub fn process_win_event(event: u32, hwnd: HWND) -> Result<()> {
        match event {
            EVENT_OBJECT_NAMECHANGE => {
                if let Some(toolbar) = SEELEN.lock().toolbar_mut() {
                    if toolbar.last_focus == Some(hwnd.0) {
                        toolbar.focus_changed(hwnd)?;
                    }
                }
            }
            EVENT_SYSTEM_FOREGROUND | EVENT_OBJECT_FOCUS => {
                if let Some(toolbar) = SEELEN.lock().toolbar_mut() {
                    toolbar.focus_changed(hwnd)?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}
