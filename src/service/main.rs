// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_management;
mod cli;
mod enviroment;
mod error;
mod hotkeys;
mod logger;
mod shutdown;
mod string_utils;
mod task_scheduler;
mod windows_api;

use cli::handle_console_client;
use error::Result;
use itertools::Itertools;
use logger::SluServiceLogger;
use shutdown::restore_native_taskbar;
use slu_ipc::{AppIpc, ServiceIpc, IPC};
use std::sync::{atomic::AtomicBool, LazyLock, OnceLock};
use string_utils::WindowsString;
use task_scheduler::TaskSchedulerHelper;
use tokio::sync::mpsc::Sender;
use windows::Win32::{
    Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
    Security::SE_TCB_NAME,
    System::Threading::CreateMutexW,
    UI::WindowsAndMessaging::SW_MINIMIZE,
};
use windows_api::WindowsApi;

use crate::{
    app_management::launch_seelen_ui,
    enviroment::{add_installation_dir_to_path, remove_installation_dir_from_path},
    hotkeys::stop_app_shortcuts,
};

pub static SERVICE_NAME: LazyLock<WindowsString> =
    LazyLock::new(|| WindowsString::from_str("slu-service"));
pub static SERVICE_DISPLAY_NAME: LazyLock<WindowsString> =
    LazyLock::new(|| WindowsString::from_str("Seelen UI Service"));

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
    WindowsApi::set_process_dpi_aware()?;
    WindowsApi::enable_privilege(SE_TCB_NAME)?;
    ServiceIpc::start(crate::cli::processing::process_action)?;

    if !AppIpc::can_stablish_connection() {
        WindowsApi::wait_for_native_shell();
        log_error!(launch_seelen_ui());
    }

    std::thread::sleep(std::time::Duration::from_secs(2));
    crate::app_management::start_app_monitoring();
    Ok(())
}

// https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw
fn is_svc_already_running() -> bool {
    unsafe {
        let session_id = WindowsApi::current_session_id();
        let mutex_name = format!("Local\\Seelen-UI-Service-Instance-{}", session_id);
        let mutex_name_wide = WindowsString::from_str(&mutex_name);

        // Try to create a named mutex specific to the current session
        let Ok(handle) = CreateMutexW(None, true, mutex_name_wide.as_pcwstr()) else {
            // Failed to create mutex, assume not running to be safe
            log::warn!("Failed to create service instance mutex, proceeding anyway");
            return false;
        };

        // if mutex existed before, another instance is already running for this session
        let last_error = GetLastError();
        if last_error == ERROR_ALREADY_EXISTS {
            return true;
        }

        // This is the first instance for this session
        // Keep the handle alive by leaking it (will be released when process exits)
        Box::leak(Box::new(handle));
        false
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    if is_local_dev() {
        let window = WindowsApi::get_console_window();
        let _ = WindowsApi::show_window(window.0 as _, SW_MINIMIZE.0);
        add_installation_dir_to_path()?;
    }

    handle_console_client().await?;
    if is_svc_already_running() {
        println!("Seelen UI Service is already running");
        return Ok(());
    }

    ASYNC_RUNTIME_HANDLE
        .set(tokio::runtime::Handle::current())
        .expect("Failed to set runtime handle");

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    EXIT_CHANNEL.set(tx).unwrap();

    SluServiceLogger::init()?;
    TaskSchedulerHelper::create_service_task()?;

    log::info!("Starting Seelen UI Service");
    log::info!("Arguments: {:?}", std::env::args().collect_vec());

    if let Err(err) = setup() {
        log::error!("Service setup failed: {:?}", err);
        // Run cleanup even on setup failure so the taskbar/hotkeys are restored
        log_error!(restore_native_taskbar());
        stop_app_shortcuts();
        return Err("Service setup failed".into());
    };

    // ===================== wait for stop signal ====================
    let exit_code = rx.recv().await.unwrap_or_default();

    // shutdown tasks:
    log_error!(restore_native_taskbar());
    stop_app_shortcuts();
    log::info!("Seelen UI Service exited with code {exit_code}");

    if is_local_dev() {
        remove_installation_dir_from_path()?;
    }

    if exit_code == 0 {
        Ok(())
    } else {
        Err("Seelen UI Service exited with error".into())
    }
}
