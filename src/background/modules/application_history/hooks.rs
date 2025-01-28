use std::ffi::OsStr;

use seelen_core::system_state::FocusedApp;

use crate::{
    error_handler::AppError,
    log_error,
    modules::start::application::START_MENU_MANAGER,
    trace_lock,
    windows_api::{window::Window, WindowsApi},
    winevent::WinEvent,
};

use super::{ApplicationHistory, APPLICATION_HISTORY};

impl ApplicationHistory {
    pub fn process_global_win_event(event: WinEvent, window: &Window) -> Result<(), AppError> {
        match event {
            WinEvent::ObjectNameChange => {
                let mut history = trace_lock!(APPLICATION_HISTORY);
                if let Ok(current) = history.current() {
                    if current.application.hwnd == window.hwnd().0 as isize {
                        log_error!(history.update_current(window.title()));
                    }
                }
            }
            WinEvent::ObjectFocus | WinEvent::SystemForeground => {
                let umid = window
                    .process()
                    .package_app_user_model_id()
                    .ok()
                    .or_else(|| window.app_user_model_id());

                let app_name = match &umid {
                    Some(umid) => {
                        if WindowsApi::is_uwp_package_id(umid) {
                            WindowsApi::get_uwp_app_info(umid)?
                                .DisplayInfo()?
                                .DisplayName()?
                                .to_string_lossy()
                        } else {
                            let shortcut = START_MENU_MANAGER
                                .load()
                                .search_shortcut_with_same_umid(umid);

                            if let Some(shortcut) = shortcut {
                                shortcut
                                    .file_stem()
                                    .unwrap_or_else(|| OsStr::new("Unknown"))
                                    .to_string_lossy()
                                    .to_string()
                            } else {
                                window
                                    .app_display_name()
                                    .unwrap_or_else(|_| String::from("Unknown"))
                            }
                        }
                    }
                    None => window
                        .app_display_name()
                        .unwrap_or_else(|_| String::from("Unknown")),
                };

                let app = FocusedApp {
                    hwnd: window.hwnd().0 as _,
                    title: window.title(),
                    name: app_name,
                    exe: window.process().program_path().ok(),
                    umid,
                };

                let mut history = trace_lock!(APPLICATION_HISTORY);
                log_error!(history.add_focused(app));
            }
            _ => {}
        }

        Ok(())
    }
}
