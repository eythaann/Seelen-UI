use std::{collections::HashMap, sync::Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{FocusedApp, UserAppWindow, UserAppWindowPreview},
};
use windows::Win32::UI::Shell::{IShellDispatch6, Shell};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::{
        apps::application::{previews::WinPreviewManager, UserAppsManager},
        input::Mouse,
    },
    windows_api::{window::Window, Com},
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

        WinPreviewManager::subscribe(|_| {
            emit_to_webviews(
                SeelenEvent::UserAppWindowsPreviewsChanged,
                WinPreviewManager::instance().previews.to_hash_map(),
            );
        });
    });
    UserAppsManager::instance()
}

#[tauri::command(async)]
pub fn get_focused_app() -> FocusedApp {
    Window::get_foregrounded().as_focused_app_information()
}

#[tauri::command(async)]
pub fn get_mouse_position() -> [i32; 2] {
    let point: seelen_core::Point = Mouse::get_cursor_pos().unwrap_or_default();
    [point.x, point.y]
}

#[tauri::command(async)]
pub fn get_user_app_windows() -> Vec<UserAppWindow> {
    get_apps_manager().interactable_windows.to_vec()
}

#[tauri::command(async)]
pub fn get_user_app_windows_previews() -> HashMap<isize, UserAppWindowPreview> {
    get_apps_manager();
    WinPreviewManager::instance().previews.to_hash_map()
}

/// This function is called show_desktop but acts more like minimize_all
#[tauri::command(async)]
pub fn show_desktop() -> Result<()> {
    Com::run_with_context(|| {
        let shell: IShellDispatch6 = Com::create_instance(&Shell)?;
        unsafe { shell.ToggleDesktop()? };
        Ok(())
    })
}
