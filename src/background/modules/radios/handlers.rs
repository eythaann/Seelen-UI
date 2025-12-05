use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{RadioDevice, RadioDeviceKind},
};
use tauri::Emitter;

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    modules::radios::manager::RadioManager,
};

/// Wrapper for RadioManager that automatically registers Tauri events on first access
/// This keeps Tauri logic separate from system logic while ensuring lazy initialization
fn get_radio_manager() -> &'static RadioManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        RadioManager::subscribe(|_event| {
            get_app_handle()
                .emit(
                    SeelenEvent::RadiosChanged,
                    RadioManager::instance().get_radios(),
                )
                .log_error();
        });
    });

    RadioManager::instance()
}

#[tauri::command(async)]
pub fn get_radios() -> Vec<RadioDevice> {
    get_radio_manager().get_radios()
}

#[tauri::command(async)]
pub fn set_radios_state(kind: RadioDeviceKind, enabled: bool) -> Result<()> {
    get_radio_manager().set_radios_state(kind, enabled)
}
