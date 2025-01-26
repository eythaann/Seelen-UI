use seelen_core::system_state::FocusedApp;

use crate::{
    error_handler::AppError, log_error, trace_lock, windows_api::window::Window, winevent::WinEvent,
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
                let title = window.title();
                let app = FocusedApp {
                    hwnd: window.hwnd().0 as _,
                    title,
                    name: window
                        .app_display_name()
                        .unwrap_or(String::from("Error on App Name")),
                    exe: window.exe().ok(),
                    umid: window
                        .process()
                        .package_app_user_model_id()
                        .ok()
                        .or_else(|| window.app_user_model_id()),
                };

                let mut history = trace_lock!(APPLICATION_HISTORY);
                log_error!(history.add_focused(app));
            }
            _ => {}
        }

        Ok(())
    }
}
