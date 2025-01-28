use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{FindWindowExA, EVENT_OBJECT_CREATE, EVENT_OBJECT_SHOW, SW_HIDE},
};

use crate::{
    error_handler::Result,
    pcstr,
    windows_api::{window::Window, WindowsApi},
    winevent::WinEvent,
};

use super::{SeelenWeg, TASKBAR_CLASS};

impl SeelenWeg {
    pub fn process_global_win_event(event: WinEvent, window: &Window) -> Result<()> {
        match event {
            WinEvent::ObjectShow | WinEvent::ObjectCreate => {
                if Self::should_be_added(window) {
                    Self::add(window)?;
                }
            }
            WinEvent::ObjectParentChange => {
                if let Some(parent) = window.parent() {
                    if Self::contains_app(window) {
                        Self::remove_hwnd(window)?;
                    }
                    if !Self::contains_app(&parent) && Self::should_be_added(&parent) {
                        Self::add(&parent)?;
                    }
                }
            }
            WinEvent::ObjectDestroy | WinEvent::ObjectHide => {
                if Self::contains_app(window) {
                    Self::remove_hwnd(window)?;
                }
            }
            WinEvent::SystemMoveSizeEnd => {
                if Self::contains_app(window) {
                    Self::update_app(window)?;
                }
            }
            WinEvent::ObjectNameChange => {
                if Self::contains_app(window) {
                    Self::update_app(window)?;
                } else if Self::should_be_added(window) {
                    Self::add(window)?;
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                // Self::set_active_window(window)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn process_individual_win_event(&mut self, event: WinEvent, origin: HWND) -> Result<()> {
        let window = Window::from(origin);
        match event {
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                self.handle_overlaped_status(origin)?;
            }
            WinEvent::ObjectLocationChange => {
                if window.hwnd() == self.window.hwnd()? {
                    self.set_position(window.monitor().handle())?;
                }
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
                }
            }
            _ => {}
        };
        Ok(())
    }

    pub fn process_raw_win_event(event: u32, origin_hwnd: HWND) -> Result<()> {
        let origin = Window::from(origin_hwnd);
        match event {
            EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE => {
                let class = origin.class();
                let parent_class = origin.parent().map(|p| p.class()).unwrap_or_default();
                if TASKBAR_CLASS
                    .iter()
                    .any(|t| t == &class || t == &parent_class)
                {
                    Self::hide_taskbar();
                    return Ok(());
                }

                if class.eq("XamlExplorerHostIslandWindow") && origin.title().is_empty() {
                    let content_hwnd = unsafe {
                        FindWindowExA(
                            origin_hwnd,
                            HWND::default(),
                            pcstr!("Windows.UI.Composition.DesktopWindowContentBridge"),
                            None,
                        )
                        .unwrap_or_default()
                    };

                    if !content_hwnd.is_invalid() {
                        let input_hwnd = unsafe {
                            FindWindowExA(
                                content_hwnd,
                                HWND::default(),
                                pcstr!("Windows.UI.Input.InputSite.WindowClass"),
                                None,
                            )
                            .unwrap_or_default()
                        };
                        if !input_hwnd.is_invalid() {
                            // can fail on volume window island
                            let _ = WindowsApi::show_window(input_hwnd, SW_HIDE);
                        }
                        // can fail on volume window island
                        let _ = WindowsApi::show_window(content_hwnd, SW_HIDE);
                        WindowsApi::show_window(origin_hwnd, SW_HIDE)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
