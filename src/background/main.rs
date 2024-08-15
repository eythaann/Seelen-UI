// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
use itertools::Itertools;
use modules::{
    cli::{
        application::{attach_console, is_just_getting_info, SEELEN_COMMAND_LINE},
        infrastructure::Client,
    },
    tray::application::ensure_tray_overflow_creation,
};
use plugins::register_plugins;
use seelen::SEELEN;
use tray::try_register_tray_icon;

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

fn setup(app: &mut tauri::App<tauri::Wry>) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("───────────────────── Starting Seelen ─────────────────────");
    Client::listen_tcp()?;

    // try it at start to avoid made it before
    log_error!(ensure_tray_overflow_creation());

    let mut seelen = unsafe { SEELEN.make_guard_unchecked() };
    seelen.init(app.handle().clone())?;

    if !tauri::is_dev() {
        log_error!(seelen.show_update_modal());

        let command = trace_lock!(SEELEN_COMMAND_LINE).clone();
        let matches = command.get_matches();
        if !matches.get_flag("silent") {
            log_error!(seelen.show_settings());
        }
    }

    seelen.start()?;
    log_error!(try_register_tray_icon(app));
    std::mem::forget(seelen);
    Ok(())
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
            log::info!("───────────────────── Exiting Seelen ─────────────────────");
            trace_lock!(SEELEN).stop()
        }
        _ => {}
    }
}

fn main() -> Result<()> {
    color_eyre::install().expect("Failed to install color_eyre");
    register_panic_hook();

    let command = trace_lock!(SEELEN_COMMAND_LINE).clone();
    let matches = match command.try_get_matches() {
        Ok(m) => m,
        // (help, --help or -h) is also managed as error
        Err(e) => {
            attach_console()?;
            e.print()?;
            return Ok(());
        }
    };

    if is_just_getting_info(&matches)? {
        return Ok(());
    }

    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    let already_running = sys.processes_by_name("seelen-ui.exe").collect_vec().len() > 1;

    if already_running {
        if let Ok(stream) = Client::connect_tcp() {
            let mut writer = BufWriter::new(stream);

            let args = std::env::args().collect_vec();
            let msg = serde_json::to_string(&args).expect("could not serialize");

            writer.write_all(msg.as_bytes()).expect("could not write");
            writer.flush().expect("could not flush");
        }
        return Ok(());
    }

    let mut app_builder = tauri::Builder::default();
    app_builder = register_plugins(app_builder);
    app_builder = register_invoke_handler(app_builder);

    let app = app_builder
        .setup(setup)
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(app_callback);
    Ok(())
}
