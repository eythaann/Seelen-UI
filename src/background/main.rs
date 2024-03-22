// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod error_handler;
mod seelen;
mod seelenweg;
mod tray;
mod webviews;
mod windows_api;

use cli::handle_cli;
use error_handler::Result;
use seelen::SEELEN;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{
    fern::colors::{Color, ColoredLevelConfig},
    Target, TargetKind,
};
use tauri_plugin_shell::ShellExt;
use tray::handle_tray_icon;
use webviews::{check_updates_window, set_windows_events};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VIRTUAL_KEY, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK,
};

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

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

#[tauri::command]
fn run_ahk_installer() {
    tauri::async_runtime::spawn(async move {
        let app = SEELEN.lock().handle().clone();
        app.shell()
            .command("static\\redis\\AutoHotKey_setup.exe")
            .spawn()
            .expect("Fail on running ahk intaller");
    });
}

#[tauri::command]
fn media_play_pause() -> Result<(), String> {
    press_key(VK_MEDIA_PLAY_PAUSE)
}

#[tauri::command]
fn media_next() -> Result<(), String> {
    press_key(VK_MEDIA_NEXT_TRACK)
}

#[tauri::command]
fn media_prev() -> Result<(), String> {
    press_key(VK_MEDIA_PREV_TRACK)
}

#[tauri::command]
fn start_seelen_shortcuts() -> Result<(), String> {
    SEELEN.lock().start_ahk_shortcuts();
    Ok(())
}

#[tauri::command]
fn kill_seelen_shortcuts() -> Result<(), String> {
    SEELEN.lock().kill_ahk_shortcuts();
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    std::panic::set_hook(Box::new(|info| {
        if let Some(s) = info.payload().downcast_ref::<String>() {
            log::error!("{}", s);
        }
    }));

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--silent"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            log::trace!("Instance Detected. Executing with: {argv:?}, from: {cwd}");
            if argv.contains(&"roulette".to_owned()) {
                return app.emit("open-seelenpad", ()).unwrap();
            }

            app.emit("open-settings", ()).unwrap();
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .with_colors(ColoredLevelConfig {
                    error: Color::Red,
                    warn: Color::Yellow,
                    debug: Color::BrightGreen,
                    info: Color::BrightBlue,
                    trace: Color::White,
                })
                .level_for("tao", log::LevelFilter::Off)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            run_ahk_installer,
            media_play_pause,
            media_next,
            media_prev,
            start_seelen_shortcuts,
            kill_seelen_shortcuts,
        ])
        .setup(|app| {
            log::info!("───────────────────── Starting Seelen ─────────────────────");

            let should_run_app = handle_cli(app)?;
            if !should_run_app {
                app.handle().exit(0);
                return Ok(());
            }

            let mut seelen = SEELEN.lock();
            seelen.init(app.handle().clone());

            seelen.ensure_folders().expect("Fail on ensuring folders");
            seelen.start_ahk_shortcuts();
            seelen.start_komorebi_manager();

            handle_tray_icon(app)?;

            set_windows_events(app)?;
            check_updates_window(app.app_handle())?;

            if false {
                // Todo(eythan) future feature should be handle by settings
                match seelen.weg().start() {
                    Ok(_) => {}
                    Err(err) => log::error!("Fail on starting SeelenWeg: {err}"),
                };
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_, event| match event {
        tauri::RunEvent::ExitRequested { api, code, .. } => {
            // prevent close background on webview windows closing
            if code.is_none() {
                api.prevent_exit();
            }
        }
        tauri::RunEvent::Exit => {
            log::info!("───────────────────── Exiting Seelen ─────────────────────");

            tauri::async_runtime::block_on(async move {
                let app = SEELEN.lock();
                if false {
                    // Todo(eythan) future feature should be handle by settings
                    app.weg().stop();
                }
                app.kill_ahk_shortcuts();
            });
        }
        _ => {}
    });

    Ok(())
}
