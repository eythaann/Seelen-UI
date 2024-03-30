use std::process::Command;

use tauri::{command, Builder, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK,
};

use crate::seelen::SEELEN;
use crate::seelenweg::handler::*;

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
fn run_ahk_installer() {
    tauri::async_runtime::spawn(async move {
        let app = SEELEN.lock().handle().clone();
        app.shell()
            .command("static\\redis\\AutoHotKey_setup.exe")
            .spawn()
            .expect("Fail on running ahk intaller");
    });
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
fn start_seelen_shortcuts() -> Result<(), String> {
    SEELEN.lock().start_ahk_shortcuts();
    Ok(())
}

#[command]
fn kill_seelen_shortcuts() -> Result<(), String> {
    SEELEN.lock().kill_ahk_shortcuts();
    Ok(())
}

#[command]
fn open_file_location(path: String) -> Result<(), String> {
    Command::new("explorer")
        .args(["/select,", &path])
        .spawn()
        .unwrap();
    Ok(())
}

pub fn register_invoke_handler(app_builder: Builder<Wry>) -> Builder<Wry> {
    app_builder.invoke_handler(tauri::generate_handler![
        // General
        open_file_location,
        // Media
        media_play_pause,
        media_next,
        media_prev,
        // AHK
        run_ahk_installer,
        start_seelen_shortcuts,
        kill_seelen_shortcuts,
        // SeelenWeg
        weg_close_app,
        weg_request_apps,
        weg_toggle_window_state,
        weg_request_update_previews,
    ])
}
