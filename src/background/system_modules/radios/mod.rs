mod application;
mod domain;

use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{RadioDevice, RadioDeviceKind},
};

use crate::{app::emit_to_webviews, error::Result};

use self::{application::RadioManager, domain::RadiosEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static RadioManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        RadioManager::subscribe(|event| match event {
            RadiosEvent::Changed => {
                emit_to_webviews(
                    SeelenEvent::RadiosChanged,
                    RadioManager::instance().radios(),
                );
            }
        });
    });
    RadioManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_radios() -> Vec<RadioDevice> {
        get_manager().radios()
    }

    pub fn set_radios_state(kind: RadioDeviceKind, enabled: bool) -> Result<()> {
        get_manager().set_enabled(kind, enabled)
    }
}
