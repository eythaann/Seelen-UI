use seelen_core::system_state::{SysTrayIcon, SysTrayIconId, SystrayIconAction};

use crate::{error::Result, modules::system_tray::application::SystemTrayManager};

#[tauri::command(async)]
pub fn get_system_tray_icons() -> Vec<SysTrayIcon> {
    SystemTrayManager::instance().icons()
}

#[tauri::command(async)]
pub fn send_system_tray_icon_action(id: SysTrayIconId, action: SystrayIconAction) -> Result<()> {
    SystemTrayManager::instance().send_action(&id, &action)?;
    Ok(())
}
