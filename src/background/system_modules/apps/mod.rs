mod application;
mod domain;

use std::{
    collections::HashMap,
    sync::{Once, OnceLock},
    time::Duration,
};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{
        FocusedApp, UserAppWindow, UserAppWindowColors, UserAppWindowPreview,
        UserAppWindowPreviewDelta, WindowIdentity,
    },
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    platform::window::Window,
    system_modules::app_previews::{AppPreviewsManager, PreviewsEvent},
    widgets::{
        permissions::{WidgetPerm, request_widget_permission},
        webview::native_window,
    },
};

pub use self::{application::UserAppsManager, domain::UserAppWinEvent};

const LEGACY_PREVIEW_SNAPSHOT_INTERVAL: Duration = Duration::from_millis(500);

/// Retained for existing subscribers. Updated widgets use the queued delta instead.
fn schedule_legacy_preview_snapshot() {
    static SENDER: OnceLock<crossbeam_channel::Sender<()>> = OnceLock::new();
    let sender = SENDER.get_or_init(|| {
        let (sender, receiver) = crossbeam_channel::bounded::<()>(1);
        std::thread::spawn(move || {
            while receiver.recv().is_ok() {
                std::thread::sleep(LEGACY_PREVIEW_SNAPSHOT_INTERVAL);
                while receiver.try_recv().is_ok() {}
                emit_to_webviews(
                    SeelenEvent::UserAppWindowsPreviewsChanged,
                    AppPreviewsManager::instance().previews(),
                );
            }
        });
        sender
    });
    let _ = sender.try_send(());
}

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
            PreviewsEvent::Captured { hwnd } => {
                let manager = AppPreviewsManager::instance();
                if let Some(preview) = manager.preview(hwnd) {
                    emit_to_webviews(
                        SeelenEvent::UserAppWindowPreviewDelta,
                        UserAppWindowPreviewDelta::Updated { hwnd, preview },
                    );
                }
                schedule_legacy_preview_snapshot();
            }
            PreviewsEvent::Cleaned { hwnd } => {
                emit_to_webviews(
                    SeelenEvent::UserAppWindowPreviewDelta,
                    UserAppWindowPreviewDelta::Removed { hwnd },
                );
                schedule_legacy_preview_snapshot();
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
    ) -> Result<seelen_core::system_state::LiveWindowPreviewState> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        if targets
            .iter()
            .any(|target| !get_manager().contains_win(&Window::from(target.id.handle)))
        {
            return Err("A preview source is no longer tracked".into());
        }
        let sources: Vec<_> = targets.iter().map(|target| target.id).collect();
        let status = AppPreviewsManager::instance().update_live(
            native_window(&webview)?.handle(),
            session,
            targets,
            opacity,
        )?;
        if status.visible {
            for hwnd in sources {
                if !status.live.contains(&hwnd) {
                    AppPreviewsManager::instance().refresh_fallback(hwnd.handle);
                }
            }
        }
        Ok(status)
    }
    pub fn clear_window_previews(webview: tauri::WebviewWindow, session: String) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        AppPreviewsManager::instance().clear_live(native_window(&webview)?.handle(), &session);
        Ok(())
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

    pub fn get_user_app_windows_previews() -> HashMap<WindowIdentity, UserAppWindowPreview> {
        get_manager();
        AppPreviewsManager::instance().previews()
    }

    pub fn get_user_app_windows_colors() -> HashMap<WindowIdentity, UserAppWindowColors> {
        get_manager();
        AppPreviewsManager::instance().colors()
    }

    /// This function is called show_desktop but acts more like minimize_all
    pub fn show_desktop(webview: tauri::WebviewWindow) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        UserAppsManager::instance().toggle_desktop()
    }
}
