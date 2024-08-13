use std::sync::atomic::{AtomicBool, Ordering};

use itertools::Itertools;
use tauri::Emitter;

use crate::{
    error_handler::Result, log_error, modules::tray::application::get_tray_icons,
    seelen::get_app_handle,
};

fn emit_tray_info() -> Result<()> {
    let handle = get_app_handle();
    let payload = get_tray_icons()?.iter().map(|t| t.info()).collect_vec();
    handle.emit("tray-info", payload)?;
    Ok(())
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_tray_events() -> Result<()> {
    if !REGISTERED.load(Ordering::Acquire) {
        log::info!("Registering tray events");
        // TODO: add event listener for tray events
        REGISTERED.store(true, Ordering::Release);
    }
    emit_tray_info()?;
    Ok(())
}

// TODO: remove when add event listener for tray events
#[tauri::command]
pub fn temp_get_by_event_tray_info() {
    log_error!(emit_tray_info());
}

#[tauri::command]
pub fn on_click_tray_icon(idx: usize) -> Result<()> {
    let icons = get_tray_icons()?;
    let icon = icons.get(idx).ok_or("tray icon index out of bounds")?;
    icon.invoke()?;
    Ok(())
}

#[tauri::command]
pub fn on_context_menu_tray_icon(idx: usize) -> Result<()> {
    let icons = get_tray_icons()?;
    let icon = icons.get(idx).ok_or("tray icon index out of bounds")?;
    icon.context_menu()?;
    Ok(())
}
