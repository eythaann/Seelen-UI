// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod apps_config;
mod cli;
mod error_handler;
mod exposed;
mod hook;
mod plugins;
mod seelen;
mod seelen_bar;
mod seelen_shell;
mod seelen_weg;
mod seelen_wm;
mod state;
mod tray;
mod utils;
mod windows_api;
mod winevent;
mod system;

use cli::handle_cli_info;
use error_handler::Result;
use exposed::register_invoke_handler;
use plugins::register_plugins;
use seelen::SEELEN;

use tray::handle_tray_icon;

use crate::cli::SEELEN_COMMAND_LINE;

fn main() -> Result<()> {
    color_eyre::install().expect("Failed to install color_eyre");
    std::panic::set_hook(Box::new(|info| {
        if let Some(s) = info.payload().downcast_ref::<String>() {
            log::error!("{}", s);
        }
    }));

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
            let mut seelen = unsafe { SEELEN.make_guard_unchecked()};
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
                seelen.stop();
            }
        }
        _ => {}
    });

    Ok(())
}
