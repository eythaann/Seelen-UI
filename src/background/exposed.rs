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

use crate::apps_config::*;
use crate::error_handler::{log_if_error, Result};
use crate::seelen::{Seelen, SEELEN};
use crate::seelen_weg::handler::*;
use crate::seelen_wm::handler::*;
use crate::system::brightness::*;
use crate::system::power::*;
use crate::utils::{is_windows_10, is_windows_11};
use crate::windows_api::WindowsApi;

fn press_key(key: VIRTUAL_KEY) -> Result<(), String> {
    let app = SEELEN.lock().handle().clone();

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
pub fn get_volume_level() -> Result<f32, String> {
    Ok(unsafe {
        WindowsApi::get_default_audio_endpoint()?
            .GetMasterVolumeLevelScalar()
            .unwrap_or_default()
    })
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
fn start_seelen_shortcuts() -> Result<(), String> {
    SEELEN.lock().start_ahk_shortcuts()?;
    Ok(())
}

#[command]
fn kill_seelen_shortcuts() -> Result<(), String> {
    SEELEN.lock().kill_ahk_shortcuts();
    Ok(())
}

#[command]
fn select_file_on_explorer(path: String) -> Result<(), String> {
    log_if_error(Command::new("explorer").args(["/select,", &path]).spawn());
    Ok(())
}

#[command]
fn open_file(path: String) -> Result<(), String> {
    log_if_error(Command::new("explorer").args([&path]).spawn());
    Ok(())
}

#[command]
fn is_dev_mode() -> bool {
    tauri::dev()
}

#[command]
fn ensure_hitboxes_zorder() {
    std::thread::spawn(|| -> Result<()> {
        let seelen = SEELEN.lock();
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
fn get_accent_color() -> String {
    let mut colorization: u32 = 0;
    let mut opaque_blend = windows::Win32::Foundation::BOOL(0);
    let _ = unsafe { DwmGetColorizationColor(&mut colorization, &mut opaque_blend) };

    let alpha = (colorization >> 24) & 0xFF;
    let red = (colorization >> 16) & 0xFF;
    let green = (colorization >> 8) & 0xFF;
    let blue = colorization & 0xFF;

    format!("#{:02X}{:02X}{:02X}{:02X}", red, green, blue, alpha)
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
    if is_windows_10() {
        WinVersion::Windows10
    } else if is_windows_11() {
        WinVersion::Windows11
    } else {
        WinVersion::Unknown
    }
}

#[command]
fn show_app_settings() {
    std::thread::spawn(|| {
        log_if_error(SEELEN.lock().show_settings());
    });
}

#[command]
fn set_auto_start(enabled: bool) {
    std::thread::spawn(move || {
        log_if_error(SEELEN.lock().set_auto_start(enabled));
    });
}

#[command]
fn get_auto_start_status() -> Result<bool, String> {
    Ok(Seelen::is_auto_start_enabled()?)
}

#[command]
fn switch_workspace(idx: u32) {
    std::thread::spawn(move || {
        winvd::switch_desktop(idx)
    });
}

pub fn register_invoke_handler(app_builder: Builder<Wry>) -> Builder<Wry> {
    app_builder.invoke_handler(tauri::generate_handler![
        // General
        is_dev_mode,
        open_file,
        select_file_on_explorer,
        get_accent_color,
        get_win_version,
        get_user_envs,
        show_app_settings,
        reload_apps_configurations,
        ensure_hitboxes_zorder,
        switch_workspace,
        // Auto Start
        set_auto_start,
        get_auto_start_status,
        // Media
        media_play_pause,
        media_next,
        media_prev,
        get_volume_level,
        set_volume_level,
        // Brightness
        get_main_monitor_brightness,
        set_main_monitor_brightness,
        // Power
        log_out,
        sleep,
        restart,
        shutdown,
        // AHK
        start_seelen_shortcuts,
        kill_seelen_shortcuts,
        // SeelenWeg
        weg_close_app,
        enum_opened_apps,
        weg_toggle_window_state,
        weg_request_update_previews,
        // Windows Manager
        set_window_position,
        bounce_handle,
        request_focus,
        complete_window_setup,
    ])
}
