use std::collections::HashMap;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{FocusedApp, UserAppWindow, UserAppWindowPreview, UserApplication},
};
use tauri::Emitter;
use windows::Win32::UI::Shell::{IShellDispatch6, Shell};

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    modules::{
        apps::application::{previews::WinPreviewManager, UserAppsManager, USER_APPS_MANAGER},
        input::Mouse,
    },
    windows_api::{window::Window, Com},
};

pub fn register_app_win_events() {
    UserAppsManager::subscribe(|_event| {
        let items = get_user_app_windows();
        get_app_handle()
            .emit(SeelenEvent::UserAppWindowsChanged, items)
            .log_error();
    });

    WinPreviewManager::subscribe(|_| {
        get_app_handle()
            .emit(
                SeelenEvent::UserAppWindowsPreviewsChanged,
                WinPreviewManager::instance().previews.to_hash_map(),
            )
            .log_error();
    });
}

#[tauri::command(async)]
pub fn get_focused_app() -> FocusedApp {
    Window::get_foregrounded().as_focused_app_information()
}

#[tauri::command(async)]
pub fn get_mouse_position() -> [i32; 2] {
    let point = Mouse::get_cursor_pos().unwrap_or_default();
    [point.x, point.y]
}

#[tauri::command(async)]
pub fn get_user_applications() -> Vec<UserApplication> {
    Vec::new()
}

#[tauri::command(async)]
pub fn get_user_app_windows() -> Vec<UserAppWindow> {
    USER_APPS_MANAGER.interactable_windows.to_vec()
}

#[tauri::command(async)]
pub fn get_user_app_windows_previews() -> HashMap<isize, UserAppWindowPreview> {
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
