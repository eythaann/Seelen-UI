use tauri::Emitter;

use crate::{
    error_handler::Result, log_error, seelen::get_app_handle, utils::spawn_named_thread,
    windows_api::MonitorEnumerator,
};

use super::{domain::PhysicalMonitor, MonitorManager};

fn _get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    let mut monitors = Vec::new();
    for m in MonitorEnumerator::get_all_v2()? {
        monitors.push(m.try_into()?);
    }
    Ok(monitors)
}

pub fn register_monitor_webview_events() -> Result<()> {
    spawn_named_thread("Monitor Manager Webview", || {
        let rx = MonitorManager::event_rx();
        while let Ok(_event) = rx.recv() {
            let handler = get_app_handle().clone();
            if let Ok(monitors) = _get_connected_monitors() {
                log_error!(handler.emit("monitors", monitors));
            }
        }
    })?;
    Ok(())
}

#[tauri::command(async)]
pub fn get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    _get_connected_monitors()
}
