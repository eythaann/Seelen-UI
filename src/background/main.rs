// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod apps_config;
mod cli;
mod error_handler;
mod exposed;
mod hook;
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
mod modules;

use cli::handle_cli_info;
use color_eyre::owo_colors::OwoColorize;
use error_handler::Result;
use exposed::register_invoke_handler;
use plugins::register_plugins;
use seelen::SEELEN;

use tray::handle_tray_icon;

use crate::{cli::SEELEN_COMMAND_LINE, error_handler::log_if_error};

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

fn main() -> Result<()> {
    color_eyre::install().expect("Failed to install color_eyre");
    register_panic_hook();

    let command = SEELEN_COMMAND_LINE.lock().clone();
    let matches = command.get_matches();
    let should_run_app = handle_cli_info(&matches);
    if !should_run_app {
        return Ok(());
    }

    let mut app_builder = tauri::Builder::default();
    app_builder = register_plugins(app_builder);
    app_builder = register_invoke_handler(app_builder);

    let app = app_builder
        .setup(move |app| {
            log::info!("───────────────────── Starting Seelen ─────────────────────");
            let mut seelen = unsafe { SEELEN.make_guard_unchecked() };
            seelen.init(app.handle().clone())?;

            handle_tray_icon(app)?;

            if !tauri::dev() {
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

    app.run(|_, event| match event {
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
                log_if_error(seelen.stop());
            }
        }
        _ => {}
    });

    Ok(())
}
