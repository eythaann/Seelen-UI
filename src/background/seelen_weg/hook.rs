use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_FOCUS, EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND
    },
};

use crate::{error_handler::Result, windows_api::WindowsApi};

use super::SeelenWeg;

impl SeelenWeg {
    pub fn process_win_event(&mut self, event: u32, hwnd: HWND) -> Result<()> {
        match event {
            EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE => {
                if "Shell_TrayWnd" == WindowsApi::get_class(hwnd)? {
                    // ensure that the taskbar is always hidden
                    SeelenWeg::hide_taskbar(true);
                }

                if SeelenWeg::is_real_window(hwnd, false) {
                    self.add_hwnd(hwnd);
                }
            }
            EVENT_OBJECT_DESTROY => {
                if self.contains_app(hwnd) {
                    self.remove_hwnd(hwnd);
                }
            }
            EVENT_OBJECT_HIDE => {
                if self.contains_app(hwnd) {
                    // We filter apps with parents but UWP apps using ApplicationFrameHost.exe are initialized without
                    // parent so we can't filter it on open event but these are immediately hidden when the ApplicationFrameHost.exe parent
                    // is assigned to the window. After that we replace the window hwnd to its parent and remove child from the list
                    let parent = WindowsApi::get_parent(hwnd);
                    if parent.0 != 0 {
                        self.replace_hwnd(hwnd, parent)?;
                    } else {
                        self.remove_hwnd(hwnd);
                    }
                }
            }
            EVENT_OBJECT_NAMECHANGE => {
                if self.contains_app(hwnd) {
                    self.update_app(hwnd);
                } else if SeelenWeg::is_real_window(hwnd, false) {
                    self.add_hwnd(hwnd);
                }
            }
            EVENT_SYSTEM_FOREGROUND | EVENT_OBJECT_FOCUS => {
                match self.contains_app(hwnd) {
                    true => self.set_active_window(hwnd)?,
                    false => self.set_active_window(HWND(0))?, // avoid rerenders on multiple unmanaged focus
                }
                self.update_status_if_needed(hwnd)?;
            }
            EVENT_OBJECT_LOCATIONCHANGE => {
                self.update_status_if_needed(hwnd)?;
            }
            _ => {}
        };
        Ok(())
    }
}
