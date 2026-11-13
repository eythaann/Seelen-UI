mod application;
mod domain;

use std::{collections::HashMap, sync::Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{FocusedApp, UserAppWindow, UserAppWindowColors, UserAppWindowPreview},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    platform::window::Window,
    system_modules::app_previews::{AppPreviewsManager, PreviewsEvent},
    widgets::permissions::{WidgetPerm, request_widget_permission},
};

pub use self::{application::UserAppsManager, domain::UserAppWinEvent};

/// Manager instance, bridging its events (and the previews ones) to the webviews on first
/// access.
fn get_manager() -> &'static UserAppsManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        UserAppsManager::subscribe(|_event| {
            emit_to_webviews(
                SeelenEvent::UserAppWindowsChanged,
                UserAppsManager::instance().interactable_windows.to_vec(),
            );
        });

        AppPreviewsManager::subscribe(|event| match event {
            PreviewsEvent::Captured { .. } | PreviewsEvent::Cleaned { .. } => {
                emit_to_webviews(
                    SeelenEvent::UserAppWindowsPreviewsChanged,
                    AppPreviewsManager::instance().previews(),
                );
            }
            PreviewsEvent::ColorsUpdated { .. } => {
                emit_to_webviews(
                    SeelenEvent::UserAppWindowsColorsChanged,
                    AppPreviewsManager::instance().colors(),
                );
            }
        });
    });
    UserAppsManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn update_window_previews(
        webview: tauri::WebviewWindow,
        session: String,
        targets: Vec<seelen_core::system_state::WindowPreviewTarget>,
        opacity: u8,
        anchor: Option<seelen_core::rect::Rect>,
    ) -> Result<seelen_core::system_state::LiveWindowPreviewState> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        if targets
            .iter()
            .any(|target| !get_manager().contains_win(&Window::from(target.hwnd)))
        {
            return Err("A preview source is no longer tracked".into());
        }
        let sources: Vec<_> = targets.iter().map(|target| target.hwnd).collect();
        let status = AppPreviewsManager::instance().update_live(
            webview.hwnd()?.0 as isize,
            session,
            targets,
            opacity,
            anchor,
        )?;
        if status.visible {
            for hwnd in sources {
                if !status.live.contains(&hwnd) {
                    AppPreviewsManager::instance().refresh_fallback(hwnd);
                }
            }
        }
        Ok(status)
    }
    pub fn clear_window_previews(webview: tauri::WebviewWindow, session: String) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        AppPreviewsManager::instance().clear_live(webview.hwnd()?.0 as isize, &session);
        Ok(())
    }
    pub fn begin_window_peek(
        webview: tauri::WebviewWindow,
        hwnd: isize,
        session: String,
        rect: seelen_core::rect::Rect,
    ) -> Result<seelen_core::system_state::WindowPeekStatus> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        if !get_manager().contains_win(&Window::from(hwnd)) {
            return Err("This window is no longer available in the dock".into());
        }
        AppPreviewsManager::instance().begin_peek(webview.hwnd()?.0 as isize, hwnd, session, rect)
    }
    pub fn poll_window_peek(
        webview: tauri::WebviewWindow,
        session: String,
        rect: Option<seelen_core::rect::Rect>,
    ) -> Result<seelen_core::system_state::WindowPeekStatus> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        Ok(AppPreviewsManager::instance().poll_peek(webview.hwnd()?.0 as isize, &session, rect))
    }
    pub fn cancel_window_peek(webview: tauri::WebviewWindow, session: String) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        AppPreviewsManager::instance().cancel_peek(webview.hwnd()?.0 as isize, &session);
        Ok(())
    }
    pub fn commit_window_peek(webview: tauri::WebviewWindow, session: String) -> Result<bool> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        AppPreviewsManager::instance().commit_peek(webview.hwnd()?.0 as isize, &session)
    }

    pub fn get_focused_app() -> FocusedApp {
        UserAppsManager::instance().focused_app()
    }

    pub fn get_mouse_position() -> [i32; 2] {
        UserAppsManager::instance().mouse_position()
    }

    pub fn get_key_state(key: String) -> Result<bool> {
        UserAppsManager::instance().is_key_down(&key)
    }

    pub fn get_user_app_windows() -> Vec<UserAppWindow> {
        get_manager().interactable_windows.to_vec()
    }

    pub fn get_user_app_windows_previews() -> HashMap<isize, UserAppWindowPreview> {
        get_manager();
        AppPreviewsManager::instance().previews()
    }

    pub fn get_user_app_windows_colors() -> HashMap<isize, UserAppWindowColors> {
        get_manager();
        AppPreviewsManager::instance().colors()
    }

    /// This function is called show_desktop but acts more like minimize_all
    pub fn show_desktop(webview: tauri::WebviewWindow) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        UserAppsManager::instance().toggle_desktop()
    }
}
