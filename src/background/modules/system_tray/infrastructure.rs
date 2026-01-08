use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{SysTrayIcon, SysTrayIconId, SystrayIconAction},
};

use crate::{
    app::emit_to_webviews, error::Result, modules::system_tray::application::SystemTrayManager,
};

fn get_system_tray_manager() -> &'static SystemTrayManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        SystemTrayManager::subscribe(|_event| {
            emit_to_webviews(
                SeelenEvent::SystemTrayChanged,
                get_system_tray_manager().icons(),
            );
        });
    });
    SystemTrayManager::instance()
}

#[tauri::command(async)]
pub fn get_system_tray_icons() -> Vec<SysTrayIcon> {
    get_system_tray_manager().icons()
}

#[tauri::command(async)]
pub fn send_system_tray_icon_action(id: SysTrayIconId, action: SystrayIconAction) -> Result<()> {
    get_system_tray_manager().send_action(&id, &action)?;
    Ok(())
}
