use windows::Win32::Foundation::HWND;

use crate::{error_handler::Result, windows_api::WindowsApi, winevent::WinEvent};

use super::FancyToolbar;

impl FancyToolbar {
    pub fn process_win_event(&mut self, event: WinEvent, origin: HWND) -> Result<()> {
        match event {
            WinEvent::ObjectNameChange => {
                if self.last_focus == Some(origin.0) {
                    self.focus_changed(origin)?;
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                self.focus_changed(origin)?;
            }
            WinEvent::SyntheticFullscreenStart => {
                let monitor = WindowsApi::monitor_from_window(self.window.hwnd()?);
                let event_monitor = WindowsApi::monitor_from_window(origin);
                if monitor == event_monitor {
                    log::trace!(
                        "Fullscreen on {} || {} || {}",
                        WindowsApi::exe(origin).unwrap_or_default(),
                        WindowsApi::get_class(origin).unwrap_or_default(),
                        WindowsApi::get_window_text(origin)
                    );
                    self.hide()?;
                }
            }
            WinEvent::SyntheticFullscreenEnd => {
                let monitor = WindowsApi::monitor_from_window(self.window.hwnd()?);
                let event_monitor = WindowsApi::monitor_from_window(origin);
                if monitor == event_monitor {
                    log::trace!(
                        "Exit Fullscreen on {} || {} || {}",
                        WindowsApi::exe(origin).unwrap_or_default(),
                        WindowsApi::get_class(origin).unwrap_or_default(),
                        WindowsApi::get_window_text(origin)
                    );
                    self.show()?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}
