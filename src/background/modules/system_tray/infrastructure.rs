use std::{path::PathBuf, sync::Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{SysTrayIcon, SysTrayIconId, SystrayIconAction},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::system_tray::application::SystemTrayManager,
    utils::{atomic_write_file, constants::SEELEN_COMMON},
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

#[tauri::command(async)]
pub fn get_system_tray_icons() -> Vec<SysTrayIcon> {
    get_system_tray_manager().icons()
}

#[tauri::command(async)]
pub fn send_system_tray_icon_action(id: SysTrayIconId, action: SystrayIconAction) -> Result<()> {
    get_system_tray_manager().send_action(&id, &action)?;
    Ok(())
}

fn pinned_tray_icons_path() -> PathBuf {
    SEELEN_COMMON.app_data_dir().join("pinned-tray-icons.json")
}

#[tauri::command(async)]
pub fn get_pinned_tray_icons() -> Result<Vec<serde_json::Value>> {
    let path = pinned_tray_icons_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[tauri::command(async)]
pub fn set_pinned_tray_icons(pinned_icons: Vec<serde_json::Value>) -> Result<()> {
    let path = pinned_tray_icons_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    atomic_write_file(&path, serde_json::to_string(&pinned_icons)?.as_bytes())?;
    Ok(())
}
