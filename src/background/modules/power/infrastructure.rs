use seelen_core::system_state::{Battery, PowerMode, PowerStatus};
use windows::Win32::System::Shutdown::{EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN, SHTDN_REASON_NONE};

use crate::{
    error_handler::Result, log_error, modules::power::application::POWER_MANAGER, trace_lock,
    windows_api::WindowsApi,
};

use super::{application::PowerManager, domain::power_status_to_serializable};

pub fn register_power_events() {
    std::thread::spawn(|| {
        trace_lock!(POWER_MANAGER)
            .init()
            .expect("Failed to initialize power manager");
    });
}

pub fn release_power_events() {
    log_error!(trace_lock!(POWER_MANAGER).release());
}

#[tauri::command(async)]
pub fn get_power_status() -> Result<PowerStatus> {
    Ok(power_status_to_serializable(
        WindowsApi::get_system_power_status()?,
    ))
}

#[tauri::command(async)]
pub fn get_power_mode() -> Result<PowerMode> {
    Ok(trace_lock!(POWER_MANAGER)
        .current_power_mode
        .clone()
        .unwrap_or(PowerMode::Unknown))
}

#[tauri::command(async)]
pub fn get_batteries() -> Result<Vec<Battery>> {
    PowerManager::get_batteries()
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
