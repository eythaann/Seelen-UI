use windows::Win32::Foundation::HWND;

use crate::{error_handler::Result, windows_api::WindowsApi, winevent::WinEvent};

use super::SeelenWeg;

impl SeelenWeg {
    pub fn process_win_event(&mut self, event: WinEvent, hwnd: HWND) -> Result<()> {
        match event {
            WinEvent::ObjectShow | WinEvent::ObjectCreate => {
                if SeelenWeg::is_real_window(hwnd, false) {
                    self.add_hwnd(hwnd);
                }
            }
            WinEvent::ObjectDestroy => {
                if self.contains_app(hwnd) {
                    self.remove_hwnd(hwnd);
                }
            }
            WinEvent::ObjectHide => {
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
            WinEvent::ObjectNameChange => {
                if self.contains_app(hwnd) {
                    self.update_app(hwnd);
                } else if SeelenWeg::is_real_window(hwnd, false) {
                    self.add_hwnd(hwnd);
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                match self.contains_app(hwnd) {
                    true => self.set_active_window(hwnd)?,
                    false => self.set_active_window(HWND(0))?, // avoid rerenders on multiple unmanaged focus
                }
                self.update_status_if_needed(hwnd)?;
            }
            WinEvent::ObjectLocationChange => {
                self.update_status_if_needed(hwnd)?;
            }
            _ => {}
        };
        Ok(())
    }
}
