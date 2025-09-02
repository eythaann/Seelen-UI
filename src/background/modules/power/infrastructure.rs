use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Battery, PowerMode, PowerStatus},
};
use tauri::Emitter;
use windows::Win32::System::Shutdown::{EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN, SHTDN_REASON_NONE};

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    modules::power::application::{PowerManagerEvent, POWER_MANAGER},
    trace_lock,
    windows_api::WindowsApi,
};

pub use super::application::PowerManager;

pub fn register_power_events() {
    let _guard = trace_lock!(POWER_MANAGER);
    PowerManager::subscribe(|event| {
        let handle = get_app_handle();
        match event {
            PowerManagerEvent::PowerStatusChanged(status) => {
                log_error!(handle.emit(SeelenEvent::PowerStatus, status));
            }
            PowerManagerEvent::BatteriesChanged(batteries) => {
                log_error!(handle.emit(SeelenEvent::BatteriesStatus, batteries));
            }
            PowerManagerEvent::PowerModeChanged(mode) => {
                log_error!(handle.emit(SeelenEvent::PowerMode, mode));
            }
        }
    });
}

pub fn release_power_events() {
    trace_lock!(POWER_MANAGER).release();
}

#[tauri::command(async)]
pub fn get_power_status() -> PowerStatus {
    trace_lock!(POWER_MANAGER).power_status.clone()
}

#[tauri::command(async)]
pub fn get_power_mode() -> PowerMode {
    trace_lock!(POWER_MANAGER).current_power_mode
}

#[tauri::command(async)]
pub fn get_batteries() -> Vec<Battery> {
    trace_lock!(POWER_MANAGER).batteries.clone()
}

#[tauri::command(async)]
pub fn log_out() {
    log_error!(WindowsApi::exit_windows(EWX_LOGOFF, SHTDN_REASON_NONE));
}

#[tauri::command(async)]
pub fn suspend() {
    log_error!(WindowsApi::set_suspend_state(false));
}

#[tauri::command(async)]
pub fn hibernate() {
    log_error!(WindowsApi::set_suspend_state(true));
}

#[tauri::command(async)]
pub fn restart() -> Result<()> {
    WindowsApi::exit_windows(EWX_REBOOT, SHTDN_REASON_NONE)?;
    Ok(())
}

#[tauri::command(async)]
pub fn shutdown() -> Result<()> {
    WindowsApi::exit_windows(EWX_SHUTDOWN, SHTDN_REASON_NONE)?;
    Ok(())
}

#[tauri::command(async)]
pub fn lock() -> Result<()> {
    WindowsApi::lock_machine()?;
    Ok(())
}
