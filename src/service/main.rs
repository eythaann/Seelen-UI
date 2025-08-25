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
use windows::Win32::{Security::SE_TCB_NAME, UI::WindowsAndMessaging::SW_MINIMIZE};
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

pub static STARTUP: AtomicBool = AtomicBool::new(false);

pub fn get_runtime_handle() -> tokio::runtime::Handle {
    ASYNC_RUNTIME_HANDLE
        .get()
        .expect("Tokio runtime was not initialized")
        .clone()
}

pub fn was_started_from_startup_action() -> bool {
    STARTUP.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn is_local_dev() -> bool {
    cfg!(dev)
}

pub fn is_development() -> bool {
    cfg!(debug_assertions)
}

pub fn exit(code: u32) {
    get_runtime_handle().spawn(async move {
        EXIT_CHANNEL.get().unwrap().send(code).await.unwrap();
    });
}

#[cfg(not(debug_assertions))]
/// will stop the service after `max_attempts` attempts
fn restart_gui_on_crash(max_attempts: usize) {
    tokio::spawn(async move {
        use crate::app_management::GUI_RESTARTED_COUNTER;
        use std::sync::atomic::Ordering;

        while GUI_RESTARTED_COUNTER.load(Ordering::SeqCst) < max_attempts {
            if !AppIpc::can_stablish_connection() {
                GUI_RESTARTED_COUNTER.fetch_add(1, Ordering::SeqCst);
                log::trace!("Seelen UI was closed unexpectedly, restarting...");

                if let Err(err) = launch_seelen_ui() {
                    log::error!("Failed to restart Seelen UI: {err}");
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        exit(1);
    });
}

#[cfg(debug_assertions)]
fn stop_service_on_seelen_ui_closed() {
    // it's ok closing the GUI before the service on development
    tokio::spawn(async {
        while AppIpc::can_stablish_connection() {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
        log::info!("Seelen UI closed, stopping the service");
        exit(0);
    });
}

pub fn setup() -> Result<()> {
    WindowsApi::set_process_dpi_aware()?;
    WindowsApi::enable_privilege(SE_TCB_NAME)?;
    ServiceIpc::start(crate::cli::processing::process_action)?;

    if was_started_from_startup_action() {
        WindowsApi::wait_for_native_shell();
        launch_seelen_ui()?;
    }

    std::thread::sleep(std::time::Duration::from_secs(2));
    #[cfg(debug_assertions)]
    {
        stop_service_on_seelen_ui_closed();
    }
    #[cfg(not(debug_assertions))]
    {
        restart_gui_on_crash(5);
    }
    Ok(())
}

fn is_svc_already_running() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    sys.processes()
        .values()
        .filter(|p| {
            p.exe()
                .is_some_and(|path| path.ends_with("slu-service.exe"))
        })
        .collect_vec()
        .len()
        > 1
}

#[tokio::main]
async fn main() -> Result<()> {
    if is_local_dev() {
        let window = WindowsApi::get_console_window();
        let _ = WindowsApi::show_window(window.0 as _, SW_MINIMIZE.0);
        add_installation_dir_to_path()?;
    }

    ASYNC_RUNTIME_HANDLE
        .set(tokio::runtime::Handle::current())
        .expect("Failed to set runtime handle");

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    EXIT_CHANNEL.set(tx).unwrap();

    handle_console_client().await?;
    if is_svc_already_running() {
        println!("Seelen UI Service is already running");
        return Ok(());
    }

    let _ = SluServiceLogger::uninstall_old_logging();
    SluServiceLogger::init()?;
    TaskSchedulerHelper::create_service_task()?;

    log::info!("Starting Seelen UI Service");
    log::info!("Arguments: {:?}", std::env::args().collect_vec());
    setup()?;

    // wait for stop signal
    let exit_code = rx.recv().await.unwrap_or_default();

    // shutdown tasks:
    restore_native_taskbar()?;
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
