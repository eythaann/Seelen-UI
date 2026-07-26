use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::PhysicalMonitor};

use crate::{app::emit_to_webviews, modules::monitors::MonitorManager};

fn get_monitor_manager() -> &'static MonitorManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        let initial = MonitorManager::instance().get_cached_data();
        log::debug!("Initial monitors: {initial:#?}");

        MonitorManager::subscribe(|event| {
            log::trace!("MonitorManagerEvent::{:?}", event);
            let monitors = MonitorManager::instance().get_cached_data();
            log::debug!("{monitors:#?}");
            emit_to_webviews(SeelenEvent::SystemMonitorsChanged, monitors);
        });
    });
    MonitorManager::instance()
}

#[tauri::command(async)]
pub fn get_connected_monitors() -> Vec<PhysicalMonitor> {
    get_monitor_manager().get_cached_data()
}
