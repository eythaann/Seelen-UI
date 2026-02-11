use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Battery, PowerMode, PowerStatus},
};
use windows::Win32::System::Shutdown::{EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN, SHTDN_REASON_NONE};

use crate::{
    app::emit_to_webviews,
    error::Result,
    log_error,
    modules::power::application::{PowerManager, PowerManagerEvent},
    utils::lock_free::TracedMutex,
    windows_api::WindowsApi,
};

/// Lazy initialization wrapper that registers Tauri events on first access
/// This keeps Tauri logic separate from system logic while ensuring lazy initialization
fn get_power_manager() -> &'static TracedMutex<PowerManager> {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        PowerManager::subscribe(|event| match event {
            PowerManagerEvent::PowerStatusChanged(status) => {
                emit_to_webviews(SeelenEvent::PowerStatus, status);
            }
            PowerManagerEvent::BatteriesChanged(batteries) => {
                emit_to_webviews(SeelenEvent::BatteriesStatus, batteries);
            }
            PowerManagerEvent::PowerModeChanged(mode) => {
                emit_to_webviews(SeelenEvent::PowerMode, mode);
            }
        });
    });
    PowerManager::instance()
}

#[tauri::command(async)]
pub fn get_power_status() -> PowerStatus {
    get_power_manager().lock().power_status.clone()
}

#[tauri::command(async)]
pub fn get_power_mode() -> PowerMode {
    get_power_manager().lock().power_mode
}

#[tauri::command(async)]
pub fn get_batteries() -> Vec<Battery> {
    get_power_manager().lock().batteries.clone()
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
