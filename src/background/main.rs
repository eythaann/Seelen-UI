// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(never_type)]

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

#[macro_use]
extern crate rust_i18n;
i18n!("src/background/i18n", fallback = "en");

use std::sync::OnceLock;

use error_handler::Result;
use exposed::register_invoke_handler;
use itertools::Itertools;
use modules::{
    cli::{
        application::{attach_console, is_just_getting_info, SEELEN_COMMAND_LINE},
        AppClient, ServiceClient,
    },
    tray::application::ensure_tray_overflow_creation,
};
use plugins::register_plugins;
use seelen::{Seelen, SEELEN};
use seelen_core::state::Settings;
use tauri::webview_version;
use tray::try_register_tray_icon;
use utils::{
    integrity::{
        check_for_webview_optimal_state, restart_as_appx, validate_webview_runtime_is_installed,
    },
    is_running_as_appx_package, was_installed_using_msix, PERFORMANCE_HELPER,
};
use windows::Win32::Security::{SE_DEBUG_NAME, SE_SHUTDOWN_NAME};
use windows_api::WindowsApi;

static APP_HANDLE: OnceLock<tauri::AppHandle<tauri::Wry>> = OnceLock::new();

pub fn is_local_dev() -> bool {
    cfg!(dev)
}

fn register_panic_hook() {
    let base_hook = std::panic::take_hook();
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
        base_hook(info);
    }));
}

fn print_initial_information() {
    let version = env!("CARGO_PKG_VERSION");
    let debug = if tauri::is_dev() { " (debug)" } else { "" };
    let local = if is_local_dev() { " (local)" } else { "" };
    log::info!(
        "───────────────────── Starting Seelen UI v{version}{local}{debug} ─────────────────────"
    );
    log::info!("Operating System: {}", os_info::get());
    log::info!("WebView2 Runtime: {:?}", webview_version());
    log::info!("Elevated        : {:?}", WindowsApi::is_elevated());
    log::info!("Locate          : {:?}", Settings::get_locale());
}

fn setup(app: &mut tauri::App<tauri::Wry>) -> Result<()> {
    print_initial_information();
    validate_webview_runtime_is_installed(app.handle())?;

    if !ServiceClient::is_running() {
        tauri::async_runtime::block_on(ServiceClient::start_service())?;
    }

    check_for_webview_optimal_state(app.handle())?;
    AppClient::listen_tcp()?;

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
        tauri::RunEvent::ExitRequested { api, code, .. } => match code {
            Some(code) => {
                // if exit code is 0 it means that the app was closed by the user
                if code == 0 {
                    log_error!(ServiceClient::emit_stop_signal());
                }
            }
            // prevent close background on webview windows closing
            None => api.prevent_exit(),
        },
        tauri::RunEvent::Exit => {
            log::info!("───────────────────── Exiting Seelen UI ─────────────────────");
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
    register_panic_hook();

    let command = trace_lock!(SEELEN_COMMAND_LINE).clone();
    let matches = match command.try_get_matches() {
        Ok(m) => m,
        Err(e) => {
            // (help, --help or -h) and other sugestions are managed as error
            attach_console()?;
            e.print()?;
            return Ok(());
        }
    };

    if is_just_getting_info(&matches)? {
        return Ok(());
    }

    if is_already_runnning() {
        return AppClient::redirect_cli_to_instance();
    }

    if was_installed_using_msix() && !is_running_as_appx_package() {
        restart_as_appx(&matches)?;
    }

    trace_lock!(PERFORMANCE_HELPER).start("setup");
    let mut app_builder = tauri::Builder::default();
    app_builder = register_plugins(app_builder);
    app_builder = register_invoke_handler(app_builder);

    let app = app_builder
        .setup(|app| {
            APP_HANDLE.set(app.handle().to_owned()).unwrap();
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
