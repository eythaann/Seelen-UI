// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod enviroment;
mod error;
mod logger;
// mod native_service;
mod string_utils;
mod task_scheduler;
mod windows_api;

use cli::{handle_cli, ServiceClient};
use crossbeam_channel::{Receiver, Sender};
use error::Result;
use itertools::Itertools;
use lazy_static::lazy_static;
use logger::SluServiceLogger;
use std::process::Command;
use string_utils::WindowsString;
use task_scheduler::TaskSchedulerHelper;
use windows::Win32::Security::SE_TCB_NAME;
use windows_api::WindowsApi;

lazy_static! {
    pub static ref SERVICE_NAME: WindowsString = WindowsString::from_str("slu-service");
    pub static ref SERVICE_DISPLAY_NAME: WindowsString =
        WindowsString::from_str("Seelen UI Service");
    static ref STOP_CHANNEL: (Sender<()>, Receiver<()>) = crossbeam_channel::unbounded();
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
    let program = std::env::current_exe()
        .unwrap()
        .with_file_name("seelen-ui.exe")
        .to_string_lossy()
        .to_string();
    // we create a link file to trick with explorer into a separated process and without elevation
    let lnk_file = WindowsApi::create_temp_shortcut(&program, "--silent")?;
    Command::new("explorer").arg(&lnk_file).status()?;
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
    WindowsApi::enable_privilege(SE_TCB_NAME)?;
    ServiceClient::listen_tcp()?;

    if !is_seelen_ui_running() {
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
    handle_cli()?;

    if is_already_runnning() {
        return Ok(());
    }

    SluServiceLogger::install()?;
    SluServiceLogger::init()?;
    TaskSchedulerHelper::create_service_task()?;

    log::info!("Starting Seelen UI Service");
    setup()?;
    STOP_CHANNEL.1.recv().unwrap();
    log::info!("Seelen UI Service stopped");
    Ok(())
}
