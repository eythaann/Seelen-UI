// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(never_type)]

mod context;
mod error_handler;
mod exposed;
mod hook;
mod instance;
mod modules;
mod plugins;
mod popups;
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

#[macro_use]
extern crate lazy_static;

use std::sync::{atomic::AtomicBool, OnceLock};

use error_handler::Result;
use exposed::register_invoke_handler;
use itertools::Itertools;
use modules::{
    cli::{application::handle_console_cli, SvcAction, TcpBgApp, TcpService},
    tray::application::ensure_tray_overflow_creation,
};
use plugins::register_plugins;
use seelen::{Seelen, SEELEN};
use tray::try_register_tray_icon;
use utils::{
    integrity::{
        check_for_webview_optimal_state, print_initial_information, register_panic_hook,
        restart_as_appx, validate_webview_runtime_is_installed,
    },
    is_running_as_appx, was_installed_using_msix, PERFORMANCE_HELPER,
};
use windows::Win32::Security::{SE_DEBUG_NAME, SE_SHUTDOWN_NAME};
use windows_api::WindowsApi;

static APP_HANDLE: OnceLock<tauri::AppHandle<tauri::Wry>> = OnceLock::new();
static SILENT: AtomicBool = AtomicBool::new(false);
static STARTUP: AtomicBool = AtomicBool::new(false);
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn is_local_dev() -> bool {
    cfg!(dev)
}

fn setup(app: &mut tauri::App<tauri::Wry>) -> Result<()> {
    print_initial_information();
    validate_webview_runtime_is_installed(app.handle())?;
    TcpBgApp::listen_tcp()?;

    if !TcpService::is_running() {
        tauri::async_runtime::block_on(TcpService::start_service())?;
    }

    check_for_webview_optimal_state(app.handle())?;

    log_error!(WindowsApi::enable_privilege(SE_SHUTDOWN_NAME));
    log_error!(WindowsApi::enable_privilege(SE_DEBUG_NAME));

    // try it at start it on open the program to avoid do it before
    log_error!(ensure_tray_overflow_creation());

    trace_lock!(SEELEN).start()?;
    log_error!(try_register_tray_icon(app));
    trace_lock!(PERFORMANCE_HELPER).end("setup");
    Ok(())
}

fn app_callback(_: &tauri::AppHandle<tauri::Wry>, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::Ready => {
            log::info!("Setup was completed, app is ready.");
        }
        tauri::RunEvent::ExitRequested { api, code, .. } => match code {
            Some(code) => {
                // if exit code is 0 it means that the app was closed by the user
                if code == 0 {
                    log_error!(TcpService::request(SvcAction::Stop));
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
    handle_console_cli()?;

    if is_already_runnning() {
        TcpBgApp::request_open_settings()?;
        return Ok(());
    }

    if was_installed_using_msix() && !is_running_as_appx() {
        restart_as_appx()?;
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
