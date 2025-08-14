// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(never_type)]

mod cli;
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
mod tauri_context;
mod tray;
mod utils;
mod virtual_desktops;
mod widget_loader;
mod widgets;
mod windows_api;
mod winevent;

#[macro_use]
extern crate rust_i18n;
i18n!("src/background/i18n", fallback = "en");

#[macro_use]
extern crate lazy_static;

use std::sync::{atomic::AtomicBool, OnceLock};

use cli::{application::handle_console_client, SelfPipe, ServicePipe};
use error_handler::Result;
use exposed::register_invoke_handler;
use itertools::Itertools;
use modules::tray::application::ensure_tray_overflow_creation;
use plugins::register_plugins;
use seelen::{Seelen, SEELEN};
use slu_ipc::messages::SvcAction;
use tray::try_register_tray_icon;
use utils::{
    integrity::{
        check_for_webview_optimal_state, print_initial_information, register_panic_hook,
        restart_as_appx, validate_webview_runtime_is_installed,
    },
    is_running_as_appx, was_installed_using_msix, PERFORMANCE_HELPER,
};

use crate::seelen::get_app_handle;

static APP_HANDLE: OnceLock<tauri::AppHandle<tauri::Wry>> = OnceLock::new();
static TOKIO_RUNTIME_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();
static SILENT: AtomicBool = AtomicBool::new(false);
static STARTUP: AtomicBool = AtomicBool::new(false);
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn is_local_dev() -> bool {
    cfg!(dev)
}

pub fn get_tokio_handle() -> &'static tokio::runtime::Handle {
    TOKIO_RUNTIME_HANDLE
        .get()
        .expect("Tokio runtime was not initialized")
}

async fn setup(app_handle: &tauri::AppHandle<tauri::Wry>) -> Result<()> {
    print_initial_information();
    validate_webview_runtime_is_installed(app_handle)?;
    SelfPipe::start_listener()?;

    if !ServicePipe::is_running() {
        ServicePipe::start_service().await?;
    }

    check_for_webview_optimal_state(app_handle)?;

    // try it at start it on open the program to avoid do it before
    log_error!(ensure_tray_overflow_creation());

    trace_lock!(SEELEN).start()?;
    log_error!(try_register_tray_icon());
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
                    log_error!(ServicePipe::request(SvcAction::Stop));
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

#[tokio::main]
async fn main() -> Result<()> {
    register_panic_hook();
    handle_console_client().await?;

    if is_already_runnning() {
        SelfPipe::request_open_settings().await?;
        return Ok(());
    }

    if was_installed_using_msix() && !is_running_as_appx() {
        restart_as_appx()?;
    }

    TOKIO_RUNTIME_HANDLE
        .set(tokio::runtime::Handle::current())
        .expect("Failed to set runtime handle");

    rust_i18n::set_locale(&seelen_core::state::Settings::get_system_language());
    trace_lock!(PERFORMANCE_HELPER).start("setup");
    let mut app_builder = tauri::Builder::default();
    app_builder = register_plugins(app_builder);
    app_builder = register_invoke_handler(app_builder);

    let app = app_builder
        .setup(|app| {
            APP_HANDLE.set(app.handle().to_owned()).unwrap();

            tokio::spawn(async move {
                let handle = get_app_handle();
                if let Err(err) = setup(handle).await {
                    log::error!("Error while setting up: {err:?}");
                    handle.exit(1);
                }
            });
            Ok(())
        })
        .build(tauri_context::get_context())
        .expect("Error while building tauri application");

    // share the current runtime with Tauri
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    app.run(app_callback);
    Ok(())
}
