use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::PhysicalMonitor};

use crate::{
    app::emit_to_webviews, error::Result, modules::monitors::MonitorManager,
    windows_api::MonitorEnumerator,
};

fn get_monitor_manager() -> &'static MonitorManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        let initial = _get_connected_monitors();
        if let Ok(monitors) = initial {
            log::debug!("Initial monitors: {monitors:#?}");
        }

        MonitorManager::subscribe(|_event| {
            if let Ok(monitors) = _get_connected_monitors() {
                log::debug!("Monitors changed: {monitors:#?}");
                emit_to_webviews(SeelenEvent::SystemMonitorsChanged, monitors);
            }
        });
    });
    MonitorManager::instance()
}

pub fn _get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    let mut monitors = Vec::new();
    for m in MonitorEnumerator::enumerate_win32()? {
        monitors.push(m.try_into()?);
    }
    Ok(monitors)
}

#[tauri::command(async)]
pub fn get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    get_monitor_manager();
    _get_connected_monitors()
}
