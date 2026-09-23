use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{SysTrayIcon, SysTrayIconId, SystrayIconAction},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::system_tray::application::{SystemTrayEvent, SystemTrayManager},
};

fn get_system_tray_manager() -> &'static SystemTrayManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        SystemTrayManager::subscribe(|_event| {
            emit_to_webviews(
                SeelenEvent::SystemTrayChanged,
                SystemTrayManager::instance().icons(),
            );
        });
    });
    SystemTrayManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_system_tray_icons() -> Vec<SysTrayIcon> {
        let manager = get_system_tray_manager();
        if manager.prune_dead_icons() {
            // keep every other tray consumer in sync, not only this caller
            SystemTrayManager::send(SystemTrayEvent::Changed);
        }
        manager.icons()
    }

    pub fn send_system_tray_icon_action(
        id: SysTrayIconId,
        action: SystrayIconAction,
    ) -> Result<()> {
        get_system_tray_manager().send_action(&id, &action)?;
        Ok(())
    }
}
