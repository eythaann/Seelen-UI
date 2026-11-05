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
