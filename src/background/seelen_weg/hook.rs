use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{FindWindowExA, EVENT_OBJECT_CREATE, EVENT_OBJECT_SHOW, SW_HIDE},
};

use crate::{error_handler::Result, pcstr, windows_api::WindowsApi, winevent::WinEvent};

use super::{SeelenWeg, TASKBAR_CLASS};

impl SeelenWeg {
    pub fn process_global_win_event(event: WinEvent, origin: HWND) -> Result<()> {
        match event {
            WinEvent::ObjectShow | WinEvent::ObjectCreate => {
                if Self::should_be_added(origin) {
                    Self::add_hwnd(origin);
                }
            }
            WinEvent::ObjectParentChange => {
                let parent = WindowsApi::get_parent(origin);
                if parent.0 != 0 && !Self::contains_app(parent) && Self::should_be_added(parent) {
                    Self::add_hwnd(parent);
                }
            }
            WinEvent::ObjectDestroy | WinEvent::ObjectHide => {
                if Self::contains_app(origin) {
                    Self::remove_hwnd(origin);
                }
            }
            WinEvent::ObjectNameChange => {
                if Self::contains_app(origin) {
                    Self::update_app(origin);
                } else if Self::should_be_added(origin) {
                    Self::add_hwnd(origin);
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                Self::set_active_window(origin)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn process_individual_win_event(&mut self, event: WinEvent, origin: HWND) -> Result<()> {
        match event {
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
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
                }
            }
            _ => {}
        };
        Ok(())
    }

    pub fn process_raw_win_event(event: u32, origin_hwnd: HWND) -> Result<()> {
        match event {
            EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE => {
                let class = WindowsApi::get_class(origin_hwnd)?;
                let parent_class =
                    WindowsApi::get_class(WindowsApi::get_parent(origin_hwnd)).unwrap_or_default();

                if TASKBAR_CLASS
                    .iter()
                    .any(|t| t == &class || t == &parent_class)
                {
                    Self::hide_taskbar();
                    return Ok(());
                }

                if class.eq("XamlExplorerHostIslandWindow")
                    && WindowsApi::get_window_text(origin_hwnd).is_empty()
                {
                    let content_hwnd = unsafe {
                        FindWindowExA(
                            origin_hwnd,
                            HWND(0),
                            pcstr!("Windows.UI.Composition.DesktopWindowContentBridge"),
                            pcstr!("DesktopWindowXamlSource"),
                        )
                    };

                    if content_hwnd.0 != 0 {
                        let input_hwnd = unsafe {
                            FindWindowExA(
                                content_hwnd,
                                HWND(0),
                                pcstr!("Windows.UI.Input.InputSite.WindowClass"),
                                None,
                            )
                        };
                        if input_hwnd.0 != 0 {
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
