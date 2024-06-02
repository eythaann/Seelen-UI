use windows::Win32::Foundation::HWND;

use crate::{error_handler::Result, windows_api::WindowsApi, winevent::WinEvent};

use super::FancyToolbar;

impl FancyToolbar {
    pub fn process_win_event(&mut self, event: WinEvent, hwnd: HWND) -> Result<()> {
        match event {
            WinEvent::ObjectNameChange => {
                if self.last_focus == Some(hwnd.0) {
                    self.focus_changed(hwnd)?;
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                self.focus_changed(hwnd)?;
            }
            WinEvent::SyntheticFullscreenStart => {
                let monitor = WindowsApi::monitor_from_window(self.window.hwnd()?);
                let event_monitor = WindowsApi::monitor_from_window(hwnd);
                if monitor == event_monitor {
                    self.hide()?;
                }
            }
            WinEvent::SyntheticFullscreenEnd => {
                let monitor = WindowsApi::monitor_from_window(self.window.hwnd()?);
                let event_monitor = WindowsApi::monitor_from_window(hwnd);
                if monitor == event_monitor {
                    self.show()?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}
