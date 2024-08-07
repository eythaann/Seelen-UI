use windows::Win32::Foundation::HWND;

use crate::{error_handler::Result, windows_api::WindowsApi, winevent::WinEvent};

use super::SeelenWeg;

impl SeelenWeg {
    pub fn process_win_event(&mut self, event: WinEvent, origin: HWND) -> Result<()> {
        match event {
            WinEvent::ObjectShow | WinEvent::ObjectCreate => {
                if SeelenWeg::is_real_window(origin, false) {
                    self.add_hwnd(origin);
                }
            }
            WinEvent::ObjectDestroy => {
                if self.contains_app(origin) {
                    self.remove_hwnd(origin);
                }
            }
            WinEvent::ObjectHide => {
                if self.contains_app(origin) {
                    // We filter apps with parents but UWP apps using ApplicationFrameHost.exe are initialized without
                    // parent so we can't filter it on open event but these are immediately hidden when the ApplicationFrameHost.exe parent
                    // is assigned to the window. After that we replace the window hwnd to its parent and remove child from the list
                    let parent = WindowsApi::get_parent(origin);
                    if parent.0 != 0 {
                        self.replace_hwnd(origin, parent)?;
                    } else {
                        self.remove_hwnd(origin);
                    }
                }
            }
            WinEvent::ObjectNameChange => {
                if self.contains_app(origin) {
                    self.update_app(origin);
                } else if SeelenWeg::is_real_window(origin, false) {
                    self.add_hwnd(origin);
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                self.set_active_window(origin)?;
                self.handle_overlaped_status(origin)?;
            }
            WinEvent::ObjectLocationChange => {
                if origin == WindowsApi::get_foreground_window() {
                    self.handle_overlaped_status(origin)?;
                }
            }
            WinEvent::SyntheticFullscreenStart(event_data) => {
                let monitor = WindowsApi::monitor_from_window(self.window.hwnd()?);
                if monitor == event_data.monitor {
                    self.hide()?;
                }
            }
            WinEvent::SyntheticFullscreenEnd(event_data) => {
                let monitor = WindowsApi::monitor_from_window(self.window.hwnd()?);
                if monitor == event_data.monitor {
                    self.show()?;
                    self.set_overlaped_status(false)?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}
