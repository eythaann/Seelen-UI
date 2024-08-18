use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use tauri::{command, Builder, Wry};
use tauri_plugin_shell::ShellExt;

use crate::error_handler::Result;
use crate::modules::input::Keyboard;
use crate::seelen::{get_app_handle, Seelen, SEELEN};
use crate::seelen_weg::handler::*;
use crate::seelen_weg::icon_extractor::extract_and_save_icon;
use crate::seelen_wm::handler::*;
use crate::state::infrastructure::*;
use crate::system::brightness::*;
use crate::utils::is_virtual_desktop_supported as virtual_desktop_supported;
use crate::{log_error, trace_lock};

use crate::modules::media::infrastructure::*;
use crate::modules::network::infrastructure::*;
use crate::modules::notifications::infrastructure::*;
use crate::modules::power::infrastructure::*;
use crate::modules::tray::infrastructure::*;

#[command]
fn start_seelen_shortcuts() {
    std::thread::spawn(|| {
        log_error!(Seelen::start_ahk_shortcuts());
    });
}

#[command]
fn kill_seelen_shortcuts() {
    std::thread::spawn(|| {
        Seelen::kill_ahk_shortcuts();
    });
}

#[command]
fn select_file_on_explorer(path: String) {
    log_error!(Command::new("explorer").args(["/select,", &path]).spawn());
}

#[command]
fn open_file(path: String) {
    log_error!(Command::new("explorer").args([&path]).spawn());
}

#[command]
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

#[command]
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

#[command]
fn is_dev_mode() -> bool {
    tauri::is_dev()
}

#[command]
pub fn get_user_envs() -> HashMap<String, String> {
    std::env::vars().collect::<HashMap<String, String>>()
}

// https://docs.rs/tauri/latest/tauri/window/struct.WindowBuilder.html#known-issues
// https://github.com/tauri-apps/wry/issues/583
#[command]
async fn show_app_settings() {
    log_error!(Seelen::show_settings());
}

#[command]
fn set_auto_start(enabled: bool) {
    std::thread::spawn(move || {
        log_error!(Seelen::set_auto_start(enabled));
    });
}

#[command]
fn get_auto_start_status() -> Result<bool, String> {
    Ok(Seelen::is_auto_start_enabled()?)
}

#[command]
fn switch_workspace(idx: u32) {
    std::thread::spawn(move || winvd::switch_desktop(idx));
}

#[command]
fn ensure_hitboxes_zorder() {
    std::thread::spawn(|| -> Result<()> {
        let seelen = trace_lock!(SEELEN);
        for monitor in seelen.monitors() {
            if let Some(toolbar) = monitor.toolbar() {
                toolbar.ensure_hitbox_zorder()?;
            }
            if let Some(weg) = monitor.weg() {
                weg.ensure_hitbox_zorder()?;
            }
        }
        Ok(())
    });
}

#[command]
fn send_keys(keys: String) -> Result<()> {
    Keyboard::new().send_keys(&keys)
}

#[command]
fn get_icon(path: String) -> Option<PathBuf> {
    extract_and_save_icon(&get_app_handle(), &path).ok()
}
#[command]
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
        ensure_hitboxes_zorder,
        send_keys,
        get_icon,
        // Seelen Settings
        set_auto_start,
        get_auto_start_status,
        state_get_themes,
        state_get_placeholders,
        state_get_weg_items,
        state_get_settings,
        state_get_specific_apps_configurations,
        state_get_wallpaper,
        state_set_wallpaper,
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
        // AHK
        start_seelen_shortcuts,
        kill_seelen_shortcuts,
        // SeelenWeg
        weg_close_app,
        weg_toggle_window_state,
        weg_request_update_previews,
        // Windows Manager
        set_window_position,
        bounce_handle,
        request_focus,
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
