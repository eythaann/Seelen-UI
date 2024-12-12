// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error_handler;
mod exposed;
mod hook;
mod instance;
mod modules;
mod plugins;
mod restoration_and_migrations;
mod seelen;
mod seelen_bar;
mod seelen_rofi;
mod seelen_wall;
mod seelen_weg;
mod seelen_wm_v2;
mod state;
mod system;
mod tray;
mod utils;
mod widget_loader;
mod windows_api;
mod winevent;

use std::{
    io::{BufWriter, Write},
    sync::OnceLock,
};

use error_handler::Result;
use exposed::register_invoke_handler;
use itertools::Itertools;
use modules::{
    cli::{
        application::{attach_console, is_just_getting_info, SEELEN_COMMAND_LINE},
        Client,
    },
    tray::application::ensure_tray_overflow_creation,
};
use plugins::register_plugins;
use seelen::{Seelen, SEELEN};
use seelen_core::state::Settings;
use tauri::webview_version;
use tauri_plugin_shell::ShellExt;
use tray::try_register_tray_icon;
use utils::{
    integrity::{check_for_webview_optimal_state, validate_webview_runtime_is_installed},
    PERFORMANCE_HELPER,
};
use windows::Win32::Security::{SE_DEBUG_NAME, SE_SHUTDOWN_NAME};
use windows_api::WindowsApi;

static APP_HANDLE: OnceLock<tauri::AppHandle<tauri::Wry>> = OnceLock::new();

fn register_panic_hook() -> Result<()> {
    std::panic::set_hook(Box::new(move |info| {
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
            cause,
            string_location
        );
    }));
    Ok(())
}

fn print_initial_information() {
    let version = env!("CARGO_PKG_VERSION");
    let dev = if tauri::is_dev() { " (dev)" } else { "" };
    log::info!("───────────────────── Starting Seelen UI v{version}{dev}─────────────────────");
    log::info!("Operating System: {}", os_info::get());
    log::info!("WebView2 Runtime: {:?}", webview_version());
    log::info!("Elevated        : {:?}", WindowsApi::is_elevated());
    log::info!("Locate          : {:?}", Settings::get_locale());
}

fn run_slu_service(app: &mut tauri::App<tauri::Wry>) -> Result<()> {
    let path = std::env::current_exe()?;
    app.shell().open(
        path.with_file_name("slu-service.exe")
            .to_string_lossy()
            .to_string(),
        None,
    )?;
    Ok(())
}

fn kill_slu_service() -> Result<()> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    let process = sys.processes().values().find(|p| {
        p.exe()
            .is_some_and(|path| path.ends_with("slu-service.exe"))
    });
    if let Some(process) = process {
        process.kill();
    }
    Ok(())
}

fn setup(app: &mut tauri::App<tauri::Wry>) -> Result<()> {
    print_initial_information();
    run_slu_service(app)?;

    validate_webview_runtime_is_installed(app.handle())?;
    check_for_webview_optimal_state(app.handle())?;

    Client::listen_tcp()?;
    APP_HANDLE
        .set(app.handle().to_owned())
        .map_err(|_| "Failed to set app handle")?;

    log_error!(WindowsApi::enable_privilege(SE_SHUTDOWN_NAME));
    log_error!(WindowsApi::enable_privilege(SE_DEBUG_NAME));

    // try it at start it on open the program to avoid do it before
    log_error!(ensure_tray_overflow_creation());

    if !tauri::is_dev() {
        let command = trace_lock!(SEELEN_COMMAND_LINE).clone();
        if !command.get_matches().get_flag("silent") {
            Seelen::show_settings()?;
        }
    }

    trace_lock!(SEELEN).start()?;
    log_error!(try_register_tray_icon(app));
    trace_lock!(PERFORMANCE_HELPER).end("setup");
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
            log::info!("───────────────────── Exiting Seelen UI ─────────────────────");
            log_error!(kill_slu_service());
            if Seelen::is_running() {
                trace_lock!(SEELEN).stop();
            }
        }
        _ => {}
    }
}

fn is_already_runnning() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    sys.processes()
        .values()
        .filter(|p| p.exe().is_some_and(|path| path.ends_with("seelen-ui.exe")))
        .collect_vec()
        .len()
        > 1
}

fn main() -> Result<()> {
    register_panic_hook()?;
    trace_lock!(PERFORMANCE_HELPER).start("setup");

    let command = trace_lock!(SEELEN_COMMAND_LINE).clone();
    let matches = match command.try_get_matches() {
        Ok(m) => m,
        // (help, --help or -h) are managed as error
        Err(e) => {
            attach_console()?;
            e.print()?;
            return Ok(());
        }
    };

    if is_just_getting_info(&matches)? {
        return Ok(());
    }

    if is_already_runnning() {
        let mut attempts = 0;
        let mut connection = Client::connect_tcp();

        while connection.is_err() && attempts < 10 {
            attempts += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
            connection = Client::connect_tcp();
        }

        if let Ok(stream) = connection {
            let mut writer = BufWriter::new(stream);

            let args = std::env::args().collect_vec();
            let msg = serde_json::to_string(&args)?;

            writer.write_all(msg.as_bytes())?;
            writer.flush()?;
            return Ok(());
        }

        // if the connection fails probably is because the app is been closing
        // so we check if the app is already running again to see if we have to
        // let the instance be created or not
        if is_already_runnning() {
            return Ok(());
        }
    }

    let mut app_builder = tauri::Builder::default();
    app_builder = register_plugins(app_builder);
    app_builder = register_invoke_handler(app_builder);

    let app = app_builder
        .setup(|app| {
            if let Err(err) = setup(app) {
                log::error!("Error while setting up: {:?}", err);
                app.handle().exit(1);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Error while building tauri application");

    app.run(app_callback);
    Ok(())
}
