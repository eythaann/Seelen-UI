// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod error;
mod logger;
mod modules;
mod platform;
#[cfg(windows)]
mod windows_api;

use std::sync::{OnceLock, atomic::AtomicBool};

use itertools::Itertools;
use slu_ipc::{AppIpc, IPC, ServiceIpc};
use tokio::sync::mpsc::Sender;

use crate::{
    cli::handle_console_client,
    error::Result,
    logger::SluServiceLogger,
    modules::{
        app_management::AppManager, hotkeys::HotkeysManager, lifecycle::LifecycleManager,
        native_wallpaper::NativeWallpaperManager, startup::StartupManager, taskbar::TaskbarManager,
    },
};

pub const SERVICE_DISPLAY_NAME: &str = "Seelen UI Service";

static ASYNC_RUNTIME_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

static EXIT_CHANNEL: OnceLock<Sender<u32>> = OnceLock::new();

pub static EXITING: AtomicBool = AtomicBool::new(false);

pub fn get_async_handler() -> tokio::runtime::Handle {
    ASYNC_RUNTIME_HANDLE
        .get()
        .expect("Tokio runtime was not initialized")
        .clone()
}

pub fn is_local_dev() -> bool {
    cfg!(dev)
}

pub fn is_development() -> bool {
    cfg!(debug_assertions)
}

pub fn exit(code: u32) {
    EXITING.store(true, std::sync::atomic::Ordering::SeqCst);
    if let Some(tx) = EXIT_CHANNEL.get() {
        let tx = tx.clone();
        get_async_handler().spawn(async move {
            if tx.send(code).await.is_err() {
                log::warn!("Exit channel closed before exit signal could be sent (code={code})");
            }
        });
    } else {
        log::error!("exit() called before EXIT_CHANNEL was initialized, forcing process exit");
        std::process::exit(code as i32);
    }
}

pub fn setup() -> Result<()> {
    platform::prepare_process()?;
    ServiceIpc::start(crate::cli::processing::process_action)?;
    // suspend/resume and end of session (shutdown, logoff, package update) listeners
    LifecycleManager::instance();

    let app_just_launched = if !AppIpc::can_stablish_connection() {
        platform::wait_for_native_shell();
        log_error!(AppManager::instance().launch_app());
        true
    } else {
        false
    };

    AppManager::instance().start_monitoring(app_just_launched);
    Ok(())
}

/// Restores what the service changed on the system: native taskbar, wallpaper and shortcuts.
fn cleanup() {
    log_error!(TaskbarManager::instance().restore());
    log_error!(NativeWallpaperManager::instance().restore());
    HotkeysManager::instance().stop();
}

#[tokio::main]
async fn main() {
    if is_local_dev() {
        platform::minimize_console();
    }

    if let Err(err) = SluServiceLogger::init() {
        let fallback = std::env::temp_dir().join("slu-service-logger-error.log");
        let _ = std::fs::write(&fallback, format!("Failed to initialize logger: {err:?}"));
        std::process::exit(1);
    }

    if let Err(err) = handle_console_client().await {
        log::error!("Failed to execute command: {err:?}");
        std::process::exit(1);
    }

    if platform::is_already_running() {
        println!("Seelen UI Service is already running");
        return;
    }

    ASYNC_RUNTIME_HANDLE
        .set(tokio::runtime::Handle::current())
        .expect("Failed to set runtime handle");

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    EXIT_CHANNEL.set(tx).unwrap();

    if let Err(err) = StartupManager::instance().register_service() {
        log::error!("Failed to create service task: {err:?}");
        std::process::exit(1);
    }

    let version = env!("CARGO_PKG_VERSION");
    let debug = if is_development() { " (debug)" } else { "" };
    let local = if is_local_dev() { " (local)" } else { "" };
    log::info!("──────────────────────────────────────────────────-");
    log::info!("Starting Seelen UI Service v{version}{local}{debug}");
    log::info!("Arguments: {:?}", std::env::args().collect_vec());

    if let Err(err) = setup() {
        log::error!("Service setup failed: {:?}", err);
        // Run cleanup even on setup failure so the taskbar/wallpaper/hotkeys are restored
        cleanup();
        std::process::exit(1);
    };

    // ===================== wait for stop signal ====================
    let exit_code = rx.recv().await.unwrap_or_default();

    // shutdown tasks:
    cleanup();
    log::info!("Seelen UI Service exited with code {exit_code}");
    std::process::exit(exit_code as i32);
}
