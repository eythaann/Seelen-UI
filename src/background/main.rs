// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod error_handler;
mod exposed;
mod hook;
mod plugins;
mod seelen;
mod seelenweg;
mod state;
mod tray;
mod utils;
mod webviews;
mod windows_api;

use cli::handle_cli;
use error_handler::Result;
use exposed::register_invoke_handler;
use plugins::register_plugins;
use seelen::SEELEN;
use tauri::Manager;

use tray::handle_tray_icon;
use webviews::{check_updates_window, set_windows_events};

fn main() -> Result<()> {
    color_eyre::install()?;

    std::panic::set_hook(Box::new(|info| {
        if let Some(s) = info.payload().downcast_ref::<String>() {
            log::error!("{}", s);
        }
    }));

    let mut app_builder = tauri::Builder::default();
    app_builder = register_plugins(app_builder);
    app_builder = register_invoke_handler(app_builder);

    app_builder
        .setup(|app| {
            log::info!("───────────────────── Starting Seelen ─────────────────────");

            let should_run_app = handle_cli(app)?;
            if !should_run_app {
                app.handle().exit(0);
                return Ok(());
            }

            let mut seelen = SEELEN.lock();
            seelen.init(app.handle().clone());
            unsafe { SEELEN.force_unlock(); }
            handle_tray_icon(app)?;
            set_windows_events(app)?;
            check_updates_window(app.app_handle())?;
            seelen.start();
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_, event| match event {
            tauri::RunEvent::ExitRequested { api, code, .. } => {
                // prevent close background on webview windows closing
                if code.is_none() {
                    api.prevent_exit();
                }
            }
            tauri::RunEvent::Exit => {
                log::info!("───────────────────── Exiting Seelen ─────────────────────");
                SEELEN.lock().stop();
            }
            _ => {}
        });

    Ok(())
}
