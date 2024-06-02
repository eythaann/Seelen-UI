use windows::Win32::Foundation::HWND;
use winvd::DesktopEvent;

use crate::{
    error_handler::Result,
    seelen::SEELEN,
    utils::{constants::FORCE_RETILING_AFTER_ADD, sleep_millis},
    windows_api::WindowsApi,
    winevent::WinEvent,
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

    pub fn process_win_event(&mut self, event: WinEvent, origin: HWND) -> Result<()> {
        match event {
            WinEvent::SystemMoveSizeStart => {
                if self.is_managed(origin) {
                    self.pseudo_pause()?;
                }
            }
            WinEvent::SystemMoveSizeEnd => {
                if self.is_managed(origin) {
                    self.force_retiling()?;
                    sleep_millis(35);
                    self.pseudo_resume()?;
                }
            }
            WinEvent::SystemMinimizeEnd => {
                if !self.is_managed(origin) && Self::should_manage(origin) {
                    self.add_hwnd(origin)?;
                }
            }
            WinEvent::SystemMinimizeStart => {
                if self.is_managed(origin) {
                    self.remove_hwnd(origin)?;
                }
            }
            WinEvent::ObjectHide => {
                if self.is_managed(origin) {
                    self.remove_hwnd(origin)?;
                }
            }
            WinEvent::ObjectDestroy => {
                let title = WindowsApi::get_window_text(origin);
                if Self::VIRTUAL_PREVIEWS.contains(&title.as_str()) {
                    self.pseudo_resume()?;
                }
                if self.is_managed(origin) {
                    self.remove_hwnd(origin)?;
                }
            }
            WinEvent::ObjectShow | WinEvent::ObjectCreate => {
                let title = WindowsApi::get_window_text(origin);
                if WindowManager::VIRTUAL_PREVIEWS.contains(&title.as_str()) {
                    self.pseudo_pause()?;
                }

                if !self.is_managed(origin) && WindowManager::should_manage(origin) {
                    self.set_active_window(origin)?;
                    if self.add_hwnd(origin)? && FORCE_RETILING_AFTER_ADD.contains(&title) {
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
            WinEvent::ObjectNameChange => {
                if !self.is_managed(origin) && WindowManager::should_manage(origin) {
                    self.set_active_window(origin)?;
                    let title = WindowsApi::get_window_text(origin);
                    if self.add_hwnd(origin)? && FORCE_RETILING_AFTER_ADD.contains(&title) {
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
            WinEvent::ObjectFocus | WinEvent::SystemForeground => {
                self.set_active_window(origin)?;
            }
            WinEvent::ObjectLocationChange => {
                if WindowsApi::is_maximized(origin) {
                    self.pseudo_pause()?;
                }
            }
            WinEvent::SyntheticFullscreenStart => self.pseudo_pause()?,
            WinEvent::SyntheticFullscreenEnd => self.pseudo_resume()?,
            _ => {}
        };
        Ok(())
    }
}
