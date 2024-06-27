// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod apps_config;
mod error_handler;
mod exposed;
mod hook;
mod modules;
mod monitor;
mod plugins;
mod seelen;
mod seelen_bar;
mod seelen_shell;
mod seelen_weg;
mod seelen_wm;
mod state;
mod system;
mod tray;
mod utils;
mod windows_api;
mod winevent;

use std::io::{BufWriter, Write};

use color_eyre::owo_colors::OwoColorize;
use error_handler::Result;
use exposed::register_invoke_handler;
use modules::{
    cli::{
        application::{handle_cli_info, SEELEN_COMMAND_LINE},
        infrastructure::Client,
    },
    tray::application::try_force_tray_overflow_creation,
};
use plugins::register_plugins;
use seelen::SEELEN;

use tray::handle_tray_icon;

fn register_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let cause = info
            .payload()
            .downcast_ref::<String>()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                info.payload()
                    .downcast_ref::<&str>()
                    .unwrap_or(&"<cause unknown>")
                    .to_string()
            });

        let mut string_location = String::from("<location unknown>");
        if let Some(location) = info.location() {
            string_location = format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }

        log::error!(
            "A panic occurred:\n  Cause: {}\n  Location: {}",
            cause.cyan(),
            string_location.purple()
        );
    }));
}

fn app_callback(_: &tauri::AppHandle<tauri::Wry>, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::ExitRequested { api, code, .. } => {
            // prevent close background on webview windows closing
            if code.is_none() {
                api.prevent_exit();
            }
        }
        tauri::RunEvent::Exit => {
            let seelen = SEELEN.lock();
            if seelen.initialized {
                log::info!("───────────────────── Exiting Seelen ─────────────────────");
                log_error!(seelen.stop());
            }
        }
        _ => {}
    }
}

fn main() -> Result<()> {
    color_eyre::install().expect("Failed to install color_eyre");
    register_panic_hook();

    let command = SEELEN_COMMAND_LINE.lock().clone();
    let matches = command.get_matches();
    let should_run_app = handle_cli_info(&matches);
    if !should_run_app {
        return Ok(());
    }

    if let Ok(stream) = Client::connect_tcp() {
        let mut writer = BufWriter::new(stream);

        let args: Vec<String> = std::env::args().collect();
        let msg = serde_json::to_string(&args).expect("could not serialize");

        writer.write_all(msg.as_bytes()).expect("could not write");
        writer.flush().expect("could not flush");
        return Ok(());
    }

    let mut app_builder = tauri::Builder::default();
    app_builder = register_plugins(app_builder);
    app_builder = register_invoke_handler(app_builder);

    let app = app_builder
        .setup(move |app| {
            log::info!("───────────────────── Starting Seelen ─────────────────────");
            Client::listen_tcp()?;
            log_error!(try_force_tray_overflow_creation());

            let mut seelen = unsafe { SEELEN.make_guard_unchecked() };
            seelen.init(app.handle().clone())?;

            handle_tray_icon(app)?;

            if !tauri::is_dev() {
                seelen.create_update_modal()?;

                if !matches.get_flag("silent") {
                    seelen.show_settings()?;
                }
            }

            seelen.start()?;
            std::mem::forget(seelen);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(app_callback);
    Ok(())
}
