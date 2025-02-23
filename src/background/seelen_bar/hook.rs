use crate::{
    error_handler::Result,
    windows_api::{window::Window, WindowsApi},
    winevent::WinEvent,
};

use super::FancyToolbar;

impl FancyToolbar {
    pub fn process_win_event(&mut self, event: WinEvent, window: &Window) -> Result<()> {
        match event {
            WinEvent::ObjectNameChange => {
                if self.last_focus == Some(window.hwnd()) {
                    self.focus_changed(window.hwnd())?;
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                self.focus_changed(window.hwnd())?;
                self.handle_overlaped_status(window.hwnd())?;
            }
            WinEvent::ObjectLocationChange => {
                if window.hwnd() == self.hwnd()? {
                    self.set_position(window.monitor().handle())?;
                }
                if window.is_focused() {
                    self.handle_overlaped_status(window.hwnd())?;
                }
            }
            WinEvent::SyntheticFullscreenStart(event_data) => {
                let monitor = WindowsApi::monitor_from_window(self.hwnd()?);
                if monitor == event_data.monitor {
                    self.hide()?;
                }
            }
            WinEvent::SyntheticFullscreenEnd(event_data) => {
                let monitor = WindowsApi::monitor_from_window(self.hwnd()?);
                if monitor == event_data.monitor {
                    self.show()?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}
