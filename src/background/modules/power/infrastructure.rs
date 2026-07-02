use std::sync::{atomic::Ordering, Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Battery, PowerMode, PowerStatus},
};
use windows::Win32::System::Shutdown::{EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN, SHTDN_REASON_NONE};

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    modules::power::application::{PowerManager, PowerManagerEvent},
    state::application::FULL_STATE,
    utils::lock_free::TracedMutex,
    widgets::manager::{GAME_MODE_ACTIVE, WIDGET_MANAGER},
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
                if FULL_STATE.load().settings.suspend_on_game_mode {
                    // Only act on real transitions. Rapid power-mode flapping (game
                    // mode toggling several times per second) would otherwise churn
                    // suspend/resume and race widget re-creation.
                    let suspended = GAME_MODE_ACTIVE.load(Ordering::Acquire);
                    match mode {
                        PowerMode::GameMode if !suspended => WIDGET_MANAGER.suspend_all(),
                        PowerMode::GameMode => {}
                        _ if suspended => WIDGET_MANAGER.resume_all().log_error(),
                        _ => {}
                    }
                }
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
    WindowsApi::exit_windows(EWX_LOGOFF, SHTDN_REASON_NONE).log_error();
}

#[tauri::command(async)]
pub fn suspend() {
    WindowsApi::set_suspend_state(false).log_error();
}

#[tauri::command(async)]
pub fn hibernate() {
    WindowsApi::set_suspend_state(true).log_error();
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
