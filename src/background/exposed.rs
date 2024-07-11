use std::collections::HashMap;
use std::process::Command;

use serde::Serialize;
use tauri::{command, Builder, Wry};
use tauri_plugin_shell::ShellExt;
use windows::core::GUID;
use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK,
};

use crate::error_handler::Result;
use crate::seelen::{get_app_handle, Seelen, SEELEN};
use crate::seelen_weg::handler::*;
use crate::seelen_wm::handler::*;
use crate::system::brightness::*;
use crate::utils::{is_windows_10, is_windows_11};
use crate::windows_api::WindowsApi;
use crate::{apps_config::*, log_error, trace_lock};

use crate::modules::network::infrastructure::*;
use crate::modules::power::infrastructure::*;
use crate::modules::tray::infrastructure::*;

fn press_key(key: VIRTUAL_KEY) -> Result<(), String> {
    let app = get_app_handle();

    app.shell()
        .command("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("(new-object -com wscript.shell).SendKeys([char]{})", key.0),
        ])
        .spawn()
        .expect("Fail on pressing key");

    Ok(())
}

#[command]
fn media_play_pause() -> Result<(), String> {
    press_key(VK_MEDIA_PLAY_PAUSE)
}

#[command]
fn media_next() -> Result<(), String> {
    press_key(VK_MEDIA_NEXT_TRACK)
}

#[command]
fn media_prev() -> Result<(), String> {
    press_key(VK_MEDIA_PREV_TRACK)
}

#[command]
pub fn media_toggle_mute() -> Result<()> {
    unsafe {
        let endpoint = WindowsApi::get_default_audio_endpoint()?;
        let muted = endpoint.GetMute()?.as_bool();
        endpoint.SetMute(!muted, &GUID::zeroed())?;
    }
    Ok(())
}

#[command]
pub fn set_volume_level(level: f32) -> Result<(), String> {
    unsafe {
        WindowsApi::get_default_audio_endpoint()?
            .SetMasterVolumeLevelScalar(level, &GUID::zeroed())
            .unwrap()
    };
    Ok(())
}

#[command]
fn refresh_state() {
    std::thread::spawn(|| {
        log_error!(trace_lock!(SEELEN).refresh_state());
    });
}

#[command]
fn start_seelen_shortcuts() {
    std::thread::spawn(|| {
        log_error!(trace_lock!(SEELEN).start_ahk_shortcuts());
    });
}

#[command]
fn kill_seelen_shortcuts() {
    std::thread::spawn(|| {
        trace_lock!(SEELEN).kill_ahk_shortcuts();
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
fn open_install_folder() {
    let exe_path = std::env::current_exe().unwrap();
    log_error!(Command::new("explorer")
        .args(["/select,", &exe_path.to_string_lossy()])
        .spawn());
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
fn is_dev_mode() -> bool {
    tauri::is_dev()
}

#[command]
fn get_accent_color() -> String {
    let mut colorization: u32 = 0;
    let mut opaque_blend = windows::Win32::Foundation::BOOL(0);
    let _ = unsafe { DwmGetColorizationColor(&mut colorization, &mut opaque_blend) };

    let _alpha = (colorization >> 24) & 0xFF;
    let red = (colorization >> 16) & 0xFF;
    let green = (colorization >> 8) & 0xFF;
    let blue = colorization & 0xFF;

    format!("#{:02X}{:02X}{:02X}", red, green, blue)
}

#[command]
pub fn get_user_envs() -> HashMap<String, String> {
    std::env::vars().collect::<HashMap<String, String>>()
}

#[derive(Serialize)]
enum WinVersion {
    Windows10,
    Windows11,
    Unknown,
}

#[command]
fn get_win_version() -> WinVersion {
    if is_windows_11() {
        WinVersion::Windows11
    } else if is_windows_10() {
        WinVersion::Windows10
    } else {
        WinVersion::Unknown
    }
}

#[command]
fn show_app_settings() {
    std::thread::spawn(|| {
        log_error!(trace_lock!(SEELEN).show_settings());
    });
}

#[command]
fn set_auto_start(enabled: bool) {
    std::thread::spawn(move || {
        log_error!(trace_lock!(SEELEN).set_auto_start(enabled));
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

pub fn register_invoke_handler(app_builder: Builder<Wry>) -> Builder<Wry> {
    app_builder.invoke_handler(tauri::generate_handler![
        // General
        refresh_state,
        is_dev_mode,
        open_file,
        open_install_folder,
        run_as_admin,
        select_file_on_explorer,
        get_accent_color,
        get_win_version,
        get_user_envs,
        show_app_settings,
        reload_apps_configurations,
        switch_workspace,
        // Auto Start
        set_auto_start,
        get_auto_start_status,
        // Media
        media_play_pause,
        media_next,
        media_prev,
        set_volume_level,
        media_toggle_mute,
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
    ])
}
