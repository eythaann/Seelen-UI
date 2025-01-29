use std::sync::atomic::{AtomicBool, Ordering};

use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{
    error_handler::Result, log_error, modules::tray::application::get_tray_icons,
    seelen::get_app_handle,
};

fn emit_tray_info() -> Result<()> {
    get_app_handle().emit(SeelenEvent::TrayInfo, get_tray_icons()?)?;
    Ok(())
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_tray_events() {
    if !REGISTERED.load(Ordering::Acquire) {
        log::trace!("Registering tray events");
        // TODO: add event listener for tray events
        REGISTERED.store(true, Ordering::Release);
    }
    // Eythan: I don't know why but it doesn't work without the thread::spawn
    // it makes a deadlock and app crashes
    std::thread::spawn(|| log_error!(emit_tray_info()));
}

// TODO: remove when add event listener for tray events
#[tauri::command(async)]
pub fn temp_get_by_event_tray_info() {
    log_error!(emit_tray_info());
}

#[tauri::command(async)]
pub fn on_click_tray_icon(key: String) -> Result<()> {
    let icons = get_tray_icons()?;
    let tray_icon = icons
        .iter()
        .find(|i| i.registry.key == key)
        .ok_or("tray icon not found")?;
    tray_icon.invoke()?;
    Ok(())
}

#[tauri::command(async)]
pub fn on_context_menu_tray_icon(key: String) -> Result<()> {
    let icons = get_tray_icons()?;
    let tray_icon = icons
        .iter()
        .find(|i| i.registry.key == key)
        .ok_or("tray icon not found")?;
    tray_icon.context_menu()?;
    Ok(())
}
