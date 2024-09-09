use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use tauri::{Builder, Wry};
use tauri_plugin_shell::ShellExt;

use crate::error_handler::Result;
use crate::log_error;
use crate::modules::input::Keyboard;
use crate::modules::virtual_desk::get_vd_manager;
use crate::seelen::{get_app_handle, Seelen};
use crate::seelen_rofi::handler::*;
use crate::seelen_weg::handler::*;
use crate::seelen_weg::icon_extractor::extract_and_save_icon;
use crate::seelen_wm::handler::*;
use crate::state::infrastructure::*;
use crate::system::brightness::*;
use crate::utils::is_virtual_desktop_supported as virtual_desktop_supported;

use crate::modules::media::infrastructure::*;
use crate::modules::network::infrastructure::*;
use crate::modules::notifications::infrastructure::*;
use crate::modules::power::infrastructure::*;
use crate::modules::system_settings::infrastructure::*;
use crate::modules::tray::infrastructure::*;

#[tauri::command(async)]
fn select_file_on_explorer(path: String) {
    log_error!(Command::new("explorer").args(["/select,", &path]).spawn());
}

#[tauri::command(async)]
fn open_file(path: String) {
    log_error!(Command::new("cmd").args(["/c", "explorer", &path]).spawn());
}

#[tauri::command(async)]
fn run_as_admin(path: String) {
    tauri::async_runtime::spawn(async move {
        let app = get_app_handle();
        log_error!(
            app.shell()
                .command("powershell")
                .args(["-Command", &format!("Start-Process '{}' -Verb runAs", path)])
                .status()
                .await
        );
    });
}

#[tauri::command(async)]
fn run(program: String, args: Vec<String>) {
    tauri::async_runtime::spawn(async move {
        log_error!(
            get_app_handle()
                .shell()
                .command(program)
                .args(args)
                .status()
                .await
        );
    });
}

#[tauri::command(async)]
fn is_dev_mode() -> bool {
    tauri::is_dev()
}

#[tauri::command(async)]
pub fn get_user_envs() -> HashMap<String, String> {
    std::env::vars().collect::<HashMap<String, String>>()
}

// https://docs.rs/tauri/latest/tauri/window/struct.WindowBuilder.html#known-issues
// https://github.com/tauri-apps/wry/issues/583
#[tauri::command(async)]
fn show_app_settings() {
    log_error!(Seelen::show_settings());
}

#[tauri::command(async)]
async fn set_auto_start(enabled: bool) -> Result<()> {
    Seelen::set_auto_start(enabled).await
}

#[tauri::command(async)]
async fn get_auto_start_status() -> Result<bool> {
    Seelen::is_auto_start_enabled().await
}

#[tauri::command(async)]
fn switch_workspace(idx: usize) -> Result<()> {
    get_vd_manager().switch_to(idx)
}

#[tauri::command(async)]
fn send_keys(keys: String) -> Result<()> {
    Keyboard::new().send_keys(&keys)
}

#[tauri::command]
fn get_icon(path: String) -> Option<PathBuf> {
    extract_and_save_icon(&get_app_handle(), &path).ok()
}

#[tauri::command(async)]
fn is_virtual_desktop_supported() -> bool {
    virtual_desktop_supported()
}

pub fn register_invoke_handler(app_builder: Builder<Wry>) -> Builder<Wry> {
    app_builder.invoke_handler(tauri::generate_handler![
        // General
        run,
        is_dev_mode,
        open_file,
        run_as_admin,
        select_file_on_explorer,
        is_virtual_desktop_supported,
        get_user_envs,
        show_app_settings,
        switch_workspace,
        send_keys,
        get_icon,
        get_system_colors,
        // Seelen Settings
        set_auto_start,
        get_auto_start_status,
        state_get_themes,
        state_get_placeholders,
        state_get_layouts,
        state_get_weg_items,
        state_get_settings,
        state_get_specific_apps_configurations,
        state_get_wallpaper,
        state_set_wallpaper,
        state_get_history,
        // Media
        media_prev,
        media_toggle_play_pause,
        media_next,
        set_volume_level,
        media_toggle_mute,
        media_set_default_device,
        // Brightness
        get_main_monitor_brightness,
        set_main_monitor_brightness,
        // Power
        log_out,
        suspend,
        restart,
        shutdown,
        // SeelenWeg
        weg_close_app,
        weg_toggle_window_state,
        weg_request_update_previews,
        // Windows Manager
        set_window_position,
        bounce_handle,
        request_focus,
        // App Launcher
        launcher_get_apps,
        // tray icons
        temp_get_by_event_tray_info,
        on_click_tray_icon,
        on_context_menu_tray_icon,
        // network
        wlan_get_profiles,
        wlan_start_scanning,
        wlan_stop_scanning,
        wlan_connect,
        wlan_disconnect,
        // notifications
        notifications_close,
        notifications_close_all,
    ])
}
