use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Battery, PowerMode, PowerStatus},
};
use windows::Win32::System::Shutdown::{
    EWX_LOGOFF, SHTDN_REASON_FLAG_PLANNED, SHTDN_REASON_MAJOR_OPERATINGSYSTEM,
    SHTDN_REASON_MINOR_UPGRADE, SHTDN_REASON_NONE, SHUTDOWN_FLAGS, SHUTDOWN_INSTALL_UPDATES,
    SHUTDOWN_POWEROFF, SHUTDOWN_REASON, SHUTDOWN_RESTART,
};

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    modules::power::application::{PowerManager, PowerManagerEvent, has_pending_os_updates},
    state::application::FULL_STATE,
    utils::lock_free::TracedMutex,
    widgets::manager::WIDGET_MANAGER,
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
                    match mode {
                        PowerMode::GameMode => WIDGET_MANAGER.suspend_all(),
                        _ => WIDGET_MANAGER.resume_all().log_error(),
                    }
                }
                emit_to_webviews(SeelenEvent::PowerMode, mode);
            }
        });
    });
    PowerManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_power_status() -> PowerStatus {
        get_power_manager().lock().power_status.clone()
    }

    pub fn get_power_mode() -> PowerMode {
        get_power_manager().lock().power_mode
    }

    pub fn get_batteries() -> Vec<Battery> {
        get_power_manager().lock().batteries.clone()
    }

    pub fn log_out() {
        WindowsApi::exit_windows(EWX_LOGOFF, SHTDN_REASON_NONE).log_error();
    }

    pub fn suspend() {
        WindowsApi::set_suspend_state(false).log_error();
    }

    pub fn hibernate() {
        WindowsApi::set_suspend_state(true).log_error();
    }

    pub fn has_pending_os_updates() -> bool {
        has_pending_os_updates()
    }

    pub fn restart(install_updates: Option<bool>) -> Result<()> {
        initiate_shutdown(SHUTDOWN_RESTART, install_updates.unwrap_or(false))
    }

    pub fn shutdown(install_updates: Option<bool>) -> Result<()> {
        initiate_shutdown(SHUTDOWN_POWEROFF, install_updates.unwrap_or(false))
    }

    pub fn lock() -> Result<()> {
        WindowsApi::lock_machine()?;
        Ok(())
    }
}

/// Updates are only applied when explicitly requested, mirroring the shell's
/// "Shut down" vs "Update and shut down" options.
fn initiate_shutdown(mut flags: SHUTDOWN_FLAGS, install_updates: bool) -> Result<()> {
    let mut reason: SHUTDOWN_REASON = SHTDN_REASON_NONE;
    if install_updates {
        flags |= SHUTDOWN_INSTALL_UPDATES;
        reason = SHTDN_REASON_FLAG_PLANNED
            | SHTDN_REASON_MAJOR_OPERATINGSYSTEM
            | SHTDN_REASON_MINOR_UPGRADE;
    }
    WindowsApi::initiate_shutdown(flags, reason)
}
