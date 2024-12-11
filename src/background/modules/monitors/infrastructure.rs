use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{
    error_handler::Result,
    log_error,
    seelen::get_app_handle,
    windows_api::{window::Window, MonitorEnumerator},
};

use super::{domain::PhysicalMonitor, MonitorManager};

fn _get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    let mut monitors = Vec::new();
    for m in MonitorEnumerator::get_all_v2()? {
        monitors.push(m.try_into()?);
    }
    Ok(monitors)
}

pub fn register_monitor_webview_events() {
    MonitorManager::subscribe(|_event| {
        if let Ok(monitors) = _get_connected_monitors() {
            log_error!(get_app_handle().emit(SeelenEvent::SystemMonitorsChanged, monitors));
        }
    });
}

#[tauri::command(async)]
pub fn get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    _get_connected_monitors()
}

#[tauri::command(async)]
pub fn get_current_monitor(window: tauri::Window) -> Result<PhysicalMonitor> {
    let monitor = Window::from(window.hwnd()?).monitor();
    monitor.try_into()
}
