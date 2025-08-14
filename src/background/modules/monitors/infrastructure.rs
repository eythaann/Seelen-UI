use seelen_core::{handlers::SeelenEvent, system_state::PhysicalMonitor};
use tauri::Emitter;

use crate::{
    error_handler::Result, log_error, modules::monitors::MONITOR_MANAGER, seelen::get_app_handle,
    windows_api::MonitorEnumerator,
};

use super::MonitorManager;

pub fn register_monitor_webview_events() -> Result<()> {
    MONITOR_MANAGER.lock().init()?;
    MonitorManager::subscribe(|_event| {
        if let Ok(monitors) = get_connected_monitors() {
            log_error!(get_app_handle().emit(SeelenEvent::SystemMonitorsChanged, monitors));
        }
    });
    Ok(())
}

#[tauri::command(async)]
pub fn get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    let mut monitors = Vec::new();
    for m in MonitorEnumerator::get_all_v2()? {
        monitors.push(m.try_into()?);
    }
    Ok(monitors)
}
