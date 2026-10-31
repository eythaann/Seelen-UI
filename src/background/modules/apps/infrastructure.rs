use std::{collections::HashMap, sync::Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{FocusedApp, UserAppWindow, UserAppWindowColors, UserAppWindowPreview},
};
use windows::Win32::UI::Shell::{IShellDispatch6, Shell};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::apps::application::{UserAppsManager, previews::WinPreviewManager},
    widgets::permissions::{WidgetPerm, request_widget_permission},
    windows_api::{Com, input::Mouse, window::Window},
};

/// Lazy initialization wrapper that registers Tauri events on first access
/// This keeps Tauri logic separate from system logic while ensuring lazy initialization
fn get_apps_manager() -> &'static UserAppsManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        UserAppsManager::subscribe(|_event| {
            emit_to_webviews(
                SeelenEvent::UserAppWindowsChanged,
                UserAppsManager::instance().interactable_windows.to_vec(),
            );
        });

        WinPreviewManager::subscribe(|event| {
            use crate::modules::apps::application::previews::WinPreviewEvent;
            match event {
                WinPreviewEvent::Captured(_) | WinPreviewEvent::Cleaned(_) => {
                    emit_to_webviews(
                        SeelenEvent::UserAppWindowsPreviewsChanged,
                        WinPreviewManager::instance().get_previews(),
                    );
                }
                WinPreviewEvent::ColorsUpdated(_) => {
                    emit_to_webviews(
                        SeelenEvent::UserAppWindowsColorsChanged,
                        WinPreviewManager::instance().get_colors(),
                    );
                }
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
            .any(|target| !get_apps_manager().contains_win(&Window::from(target.hwnd)))
        {
            return Err("A preview source is no longer tracked".into());
        }
        let sources: Vec<_> = targets.iter().map(|target| target.hwnd).collect();
        let status = super::application::live_previews::LivePreviewManager::instance().update(
            webview.hwnd()?.0 as isize,
            session,
            targets,
            opacity,
            anchor,
        )?;
        if status.visible {
            for hwnd in sources {
                if !status.live.contains(&hwnd) {
                    WinPreviewManager::instance().refresh_fallback(hwnd);
                }
            }
        }
        Ok(status)
    }
    pub fn clear_window_previews(webview: tauri::WebviewWindow, session: String) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        super::application::live_previews::LivePreviewManager::instance()
            .clear(webview.hwnd()?.0 as isize, &session);
        Ok(())
    }
    pub fn begin_window_peek(
        webview: tauri::WebviewWindow,
        hwnd: isize,
        session: String,
        rect: seelen_core::rect::Rect,
    ) -> Result<seelen_core::system_state::WindowPeekStatus> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        if !get_apps_manager().contains_win(&Window::from(hwnd)) {
            return Err("This window is no longer available in the dock".into());
        }
        super::application::window_peek::WindowPeekManager::instance().begin(
            webview.hwnd()?.0 as isize,
            hwnd,
            session,
            rect,
        )
    }
    pub fn poll_window_peek(
        webview: tauri::WebviewWindow,
        session: String,
        rect: Option<seelen_core::rect::Rect>,
    ) -> Result<seelen_core::system_state::WindowPeekStatus> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        Ok(
            super::application::window_peek::WindowPeekManager::instance().poll(
                webview.hwnd()?.0 as isize,
                &session,
                rect,
            ),
        )
    }
    pub fn cancel_window_peek(webview: tauri::WebviewWindow, session: String) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        super::application::window_peek::WindowPeekManager::instance()
            .cancel(webview.hwnd()?.0 as isize, &session);
        Ok(())
    }
    pub fn commit_window_peek(webview: tauri::WebviewWindow, session: String) -> Result<bool> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        super::application::window_peek::WindowPeekManager::instance()
            .commit(webview.hwnd()?.0 as isize, &session)
    }
    pub fn get_focused_app() -> FocusedApp {
        Window::get_foregrounded().as_focused_app_information()
    }

    pub fn get_mouse_position() -> [i32; 2] {
        let point = Mouse::get_cursor_pos().unwrap_or_default();
        [point.x, point.y]
    }

    pub fn get_key_state(key: String) -> Result<bool> {
        use win_hotkeys::VKey;
        use win_hotkeys::state::KeyboardState;

        let key = VKey::from_keyname(&key)?;
        if key == VKey::Menu {
            return Ok(KeyboardState::async_is_key_down(VKey::Menu.to_vk_code())
                || KeyboardState::async_is_key_down(VKey::LMenu.to_vk_code())
                || KeyboardState::async_is_key_down(VKey::RMenu.to_vk_code()));
        }

        Ok(KeyboardState::async_is_key_down(key.to_vk_code()))
    }

    pub fn get_user_app_windows() -> Vec<UserAppWindow> {
        get_apps_manager().interactable_windows.to_vec()
    }

    pub fn get_user_app_windows_previews() -> HashMap<isize, UserAppWindowPreview> {
        get_apps_manager();
        WinPreviewManager::instance().get_previews()
    }

    pub fn get_user_app_windows_colors() -> HashMap<isize, UserAppWindowColors> {
        get_apps_manager();
        WinPreviewManager::instance().get_colors()
    }

    /// This function is called show_desktop but acts more like minimize_all
    pub fn show_desktop(webview: tauri::WebviewWindow) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ManageAppWindows)?;
        Com::run_with_context(|| {
            let shell: IShellDispatch6 = Com::create_instance(&Shell)?;
            unsafe { shell.ToggleDesktop()? };
            Ok(())
        })
    }
}
