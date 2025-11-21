use seelen_core::{
    handlers::SeelenEvent,
    system_state::{FocusedApp, UserAppWindow, UserApplication},
};
use tauri::Emitter;
use windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE;

use crate::{
    app::get_app_handle,
    error::{ErrorMap, ResultLogExt},
    modules::{
        apps::application::{UserAppsManager, USER_APPS_MANAGER},
        input::Mouse,
    },
    windows_api::window::Window,
};

pub fn register_app_win_events() {
    UserAppsManager::subscribe(|_event| {
        let items = get_user_app_windows();
        get_app_handle()
            .emit(SeelenEvent::UserAppWindowsChanged, items)
            .wrap_error()
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
    [point.x(), point.y()]
}

#[tauri::command(async)]
pub fn get_user_applications() -> Vec<UserApplication> {
    Vec::new()
}

#[tauri::command(async)]
pub fn get_user_app_windows() -> Vec<UserAppWindow> {
    USER_APPS_MANAGER.interactable_windows.to_vec()
}

/// This function is called show_desktop but acts more like minimize_all
#[tauri::command(async)]
pub fn show_desktop() {
    USER_APPS_MANAGER.interactable_windows.for_each(|data| {
        let win = Window::from(data.hwnd);
        if !win.is_minimized() {
            win.show_window_async(SW_MINIMIZE).log_error();
        }
    });
}
