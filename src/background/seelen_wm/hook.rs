use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_FOCUS, EVENT_OBJECT_HIDE,
        EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW,
        EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
        EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART,
    },
};
use winvd::DesktopEvent;

use crate::{
    error_handler::Result,
    seelen::SEELEN,
    utils::{constants::FORCE_RETILING_AFTER_ADD, sleep_millis},
    windows_api::WindowsApi,
};

use super::WindowManager;

impl WindowManager {
    pub fn process_vd_event(&mut self, event: &DesktopEvent) -> Result<()> {
        match event {
            DesktopEvent::DesktopChanged { new, old: _ } => {
                self.discard_reservation()?;
                self.set_active_workspace(format!("{:?}", new.get_id()?))?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn process_win_event(&mut self, event: u32, hwnd: HWND) -> Result<()> {
        match event {
            EVENT_SYSTEM_MOVESIZESTART => {
                if self.is_managed(hwnd) {
                    self.pseudo_pause()?;
                }
            }
            EVENT_SYSTEM_MOVESIZEEND => {
                if self.is_managed(hwnd) {
                    self.force_retiling()?;
                    sleep_millis(35);
                    self.pseudo_resume()?;
                }
            }
            EVENT_SYSTEM_MINIMIZEEND => {
                if !self.is_managed(hwnd) && Self::should_manage(hwnd) {
                    self.add_hwnd(hwnd)?;
                }
            }
            EVENT_SYSTEM_MINIMIZESTART => {
                if self.is_managed(hwnd) {
                    self.remove_hwnd(hwnd)?;
                }
            }
            EVENT_OBJECT_HIDE => {
                if self.is_managed(hwnd) {
                    self.remove_hwnd(hwnd)?;
                }
            }
            EVENT_OBJECT_DESTROY => {
                let title = WindowsApi::get_window_text(hwnd);
                if Self::VIRTUAL_PREVIEWS.contains(&title.as_str()) {
                    self.pseudo_resume()?;
                }
                if self.is_managed(hwnd) {
                    self.remove_hwnd(hwnd)?;
                }
            }
            EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE => {
                let title = WindowsApi::get_window_text(hwnd);
                if WindowManager::VIRTUAL_PREVIEWS.contains(&title.as_str()) {
                    self.pseudo_pause()?;
                }

                if !self.is_managed(hwnd) && WindowManager::should_manage(hwnd) {
                    self.set_active_window(hwnd)?;
                    if self.add_hwnd(hwnd)? && FORCE_RETILING_AFTER_ADD.contains(&title) {
                        // Todo search a better way to do this
                        std::thread::spawn(|| -> Result<()> {
                            sleep_millis(250);
                            if let Some(monitor) = SEELEN.lock().focused_monitor() {
                                monitor.wm().as_ref().unwrap().force_retiling()?
                            }
                            Ok(())
                        });
                    };
                }
            }
            EVENT_OBJECT_NAMECHANGE => {
                if !self.is_managed(hwnd) && WindowManager::should_manage(hwnd) {
                    self.set_active_window(hwnd)?;
                    let title = WindowsApi::get_window_text(hwnd);
                    if self.add_hwnd(hwnd)? && FORCE_RETILING_AFTER_ADD.contains(&title) {
                        // Todo search a better way to do this
                        std::thread::spawn(|| -> Result<()> {
                            sleep_millis(250);
                            if let Some(monitor) = SEELEN.lock().focused_monitor() {
                                monitor.wm().as_ref().unwrap().force_retiling()?
                            }
                            Ok(())
                        });
                    };
                }
            }
            EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND => {
                self.set_active_window(hwnd)?;
            }
            EVENT_OBJECT_LOCATIONCHANGE => {
                if WindowsApi::is_maximized(hwnd) {
                    self.pseudo_pause()?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}
