// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod enviroment;
mod error;
mod logger;
mod shutdown;
mod string_utils;
mod task_scheduler;
mod windows_api;

use cli::{handle_console_client, TcpService};
use crossbeam_channel::{Receiver, Sender};
use enviroment::was_installed_using_msix;
use error::Result;
use itertools::Itertools;
use lazy_static::lazy_static;
use logger::SluServiceLogger;
use shutdown::restore_native_taskbar;
use std::{process::Command, sync::atomic::AtomicBool};
use string_utils::WindowsString;
use task_scheduler::TaskSchedulerHelper;
use windows::Win32::{
    Security::SE_TCB_NAME,
    UI::{Shell::FOLDERID_LocalAppData, WindowsAndMessaging::SW_MINIMIZE},
};
use windows_api::WindowsApi;

lazy_static! {
    pub static ref SERVICE_NAME: WindowsString = WindowsString::from_str("slu-service");
    pub static ref SERVICE_DISPLAY_NAME: WindowsString =
        WindowsString::from_str("Seelen UI Service");
    static ref STOP_CHANNEL: (Sender<()>, Receiver<()>) = crossbeam_channel::unbounded();
}

pub static STARTUP: AtomicBool = AtomicBool::new(false);

pub fn was_started_from_startup_action() -> bool {
    STARTUP.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn is_local_dev() -> bool {
    cfg!(dev)
}

pub fn is_development() -> bool {
    cfg!(debug_assertions)
}

pub fn stop() {
    STOP_CHANNEL.0.send(()).unwrap();
}

fn is_seelen_ui_running() -> bool {
    let mut system = sysinfo::System::new();
    system.refresh_processes();
    system
        .processes()
        .values()
        .any(|p| p.exe().is_some_and(|path| path.ends_with("seelen-ui.exe")))
}

fn launch_seelen_ui() -> Result<()> {
    let app_path = if was_installed_using_msix() {
        WindowsApi::known_folder(FOLDERID_LocalAppData)?
            .join("Microsoft\\WindowsApps\\seelen-ui.exe")
    } else {
        std::env::current_exe()?.with_file_name("seelen-ui.exe")
    };

    let mut args = Vec::new();
    if was_started_from_startup_action() {
        args.push("--startup".to_string());
    }

    let lnk_file = WindowsApi::create_temp_shortcut(&app_path, &args.join(" "))?;
    // start it using explorer to spawn it as unelevated
    Command::new("C:\\Windows\\explorer.exe")
        .arg(&lnk_file)
        .status()?;
    std::fs::remove_file(&lnk_file)?;
    Ok(())
}

#[cfg(not(debug_assertions))]
/// will stop the service after `max_attempts` attempts
fn restart_gui_on_crash(max_attempts: u32) {
    std::thread::spawn(move || {
        let mut attempts = 0;
        while attempts < max_attempts {
            if !is_seelen_ui_running() {
                attempts += 1;
                launch_seelen_ui().expect("Failed to launch Seelen UI");
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        stop();
    });
}

#[cfg(debug_assertions)]
fn stop_service_on_seelen_ui_closed() {
    std::thread::spawn(move || {
        while is_seelen_ui_running() {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        stop();
    });
}

pub fn setup() -> Result<()> {
    WindowsApi::set_process_dpi_aware()?;
    WindowsApi::enable_privilege(SE_TCB_NAME)?;
    TcpService::listen_tcp()?;

    if was_started_from_startup_action() {
        WindowsApi::wait_for_native_shell();
        launch_seelen_ui()?;
    }

    #[cfg(debug_assertions)]
    {
        stop_service_on_seelen_ui_closed();
    }
    #[cfg(not(debug_assertions))]
    {
        restart_gui_on_crash(3);
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

fn main() -> Result<()> {
    if is_local_dev() {
        WindowsApi::show_window(WindowsApi::get_console_window().0 as _, SW_MINIMIZE.0)?;
    }
    handle_console_client()?;
    if is_svc_already_running() {
        return Ok(());
    }

    let _ = SluServiceLogger::uninstall_old_logging();
    SluServiceLogger::init()?;
    TaskSchedulerHelper::create_service_task()?;

    log::info!("Starting Seelen UI Service");
    log::info!("Arguments: {:?}", std::env::args().collect_vec());
    setup()?;

    // wait for stop signal
    STOP_CHANNEL.1.recv().unwrap();
    // shutdown tasks:
    restore_native_taskbar()?;
    log::info!("Seelen UI Service stopped");
    Ok(())
}
