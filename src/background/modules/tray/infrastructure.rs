use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{
    error_handler::Result,
    log_error,
    modules::tray::application::{TrayIconManager, TRAY_ICON_MANAGER},
    seelen::get_app_handle,
    trace_lock,
};

use super::domain::TrayIcon;

pub fn register_tray_icons_events() {
    log_error!(trace_lock!(TRAY_ICON_MANAGER).init());
    TrayIconManager::subscribe(|_event| {
        let manager = trace_lock!(TRAY_ICON_MANAGER);
        log_error!(get_app_handle().emit(SeelenEvent::TrayInfo, &manager.icons));
    });
}

#[tauri::command(async)]
pub fn get_tray_icons() -> Vec<TrayIcon> {
    trace_lock!(TRAY_ICON_MANAGER).icons.clone()
}

#[tauri::command(async)]
pub fn on_click_tray_icon(key: String) -> Result<()> {
    let manager = trace_lock!(TRAY_ICON_MANAGER);
    let tray_icon = manager
        .automation_by_key
        .get(&key)
        .ok_or("tray icon not found")?;
    tray_icon.invoke()?;
    Ok(())
}

#[tauri::command(async)]
pub fn on_context_menu_tray_icon(key: String) -> Result<()> {
    let manager = trace_lock!(TRAY_ICON_MANAGER);
    let tray_icon = manager
        .automation_by_key
        .get(&key)
        .ok_or("tray icon not found")?;
    tray_icon.context_menu()?;
    Ok(())
}
