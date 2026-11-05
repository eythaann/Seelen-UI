mod application;
mod domain;

use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Battery, PowerMode, PowerStatus},
};

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
};

pub use self::{application::PowerManager, domain::PowerEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static PowerManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        PowerManager::subscribe(|event| match event {
            PowerEvent::StatusChanged { status } => {
                emit_to_webviews(SeelenEvent::PowerStatus, status);
            }
            PowerEvent::BatteriesChanged { batteries } => {
                emit_to_webviews(SeelenEvent::BatteriesStatus, batteries);
            }
            PowerEvent::ModeChanged { mode } => {
                emit_to_webviews(SeelenEvent::PowerMode, mode);
            }
        });
    });
    PowerManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_power_status() -> PowerStatus {
        get_manager().status()
    }

    pub fn get_power_mode() -> PowerMode {
        get_manager().mode()
    }

    pub fn get_batteries() -> Vec<Battery> {
        get_manager().batteries()
    }

    pub fn has_pending_os_updates() -> bool {
        get_manager().has_pending_os_updates()
    }

    pub fn log_out() {
        get_manager().log_out().log_error();
    }

    pub fn suspend() {
        get_manager().suspend().log_error();
    }

    pub fn hibernate() {
        get_manager().hibernate().log_error();
    }

    pub fn lock() -> Result<()> {
        get_manager().lock()
    }

    pub fn restart(install_updates: Option<bool>) -> Result<()> {
        get_manager().restart(install_updates.unwrap_or(false))
    }

    pub fn shutdown(install_updates: Option<bool>) -> Result<()> {
        get_manager().shutdown(install_updates.unwrap_or(false))
    }
}
