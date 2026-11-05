mod application;
mod domain;

use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{SysTrayIcon, SystrayIconAction},
};

use crate::{app::emit_to_webviews, error::Result};

pub use self::application::SystemTrayManager;
use self::domain::SystemTrayEvent;

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static SystemTrayManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        SystemTrayManager::subscribe(|event| match event {
            SystemTrayEvent::Changed => {
                emit_to_webviews(
                    SeelenEvent::SystemTrayChanged,
                    SystemTrayManager::instance().icons(),
                );
            }
        });
    });
    SystemTrayManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_system_tray_icons() -> Vec<SysTrayIcon> {
        get_manager().icons()
    }

    pub fn send_system_tray_icon_action(id: String, action: SystrayIconAction) -> Result<()> {
        get_manager().send_action(&id, &action)
    }

    pub fn set_system_tray_icon_promoted(id: String, promoted: bool) -> Result<()> {
        get_manager().set_promoted(&id, promoted)
    }

    pub fn set_system_tray_icons_order(ids: Vec<String>) -> Result<()> {
        get_manager().set_order(&ids)
    }
}
