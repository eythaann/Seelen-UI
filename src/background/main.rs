// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod error_handler;
mod exposed;
mod hook;
mod k_killer;
mod plugins;
mod seelen;
mod seelen_bar;
mod seelen_shell;
mod seelenweg;
mod state;
mod tray;
mod utils;
mod windows_api;
mod winevent;

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
            let mut seelen = SEELEN.lock();
            seelen.init(app.handle().clone());
            unsafe {
                SEELEN.force_unlock();
            }
            handle_tray_icon(app)?;
            seelen.create_update_modal()?;

            if !tauri::dev() && !matches.get_flag("silent") {
                seelen.show_settings()?;
            }

            seelen.start();
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
            let seleen = SEELEN.lock();
            if seleen.initialized {
                log::info!("───────────────────── Exiting Seelen ─────────────────────");
                seleen.stop();
            }
        }
        _ => {}
    });

    Ok(())
}
