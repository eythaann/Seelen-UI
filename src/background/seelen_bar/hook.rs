use windows::Win32::Foundation::HWND;

use crate::{
    error_handler::Result,
    windows_api::{window::Window, WindowsApi},
    winevent::WinEvent,
};

use super::FancyToolbar;

impl FancyToolbar {
    pub fn process_win_event(&mut self, event: WinEvent, origin: HWND) -> Result<()> {
        let window = Window::from(origin);
        match event {
            WinEvent::ObjectNameChange => {
                if self.last_focus == Some(origin) {
                    self.focus_changed(origin)?;
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                self.focus_changed(origin)?;
                self.handle_overlaped_status(origin)?;
            }
            WinEvent::ObjectLocationChange => {
                if window.hwnd() == self.hwnd()? {
                    self.set_position(window.monitor().handle())?;
                }
                if origin == WindowsApi::get_foreground_window() {
                    self.handle_overlaped_status(origin)?;
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
