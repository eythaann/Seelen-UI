use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{MonitorId, PhysicalMonitor},
};

use crate::{
    app::emit_to_webviews, error::Result, modules::monitors::MonitorManager,
    windows_api::MonitorEnumerator,
};

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

impl crate::tauri_handlers::Handlers {
    pub fn get_connected_monitors() -> Vec<PhysicalMonitor> {
        get_monitor_manager().get_cached_data()
    }

    pub fn set_monitor_hdr(id: MonitorId, state: bool) -> Result<()> {
        let monitor = MonitorEnumerator::enumerate_win32()?
            .into_iter()
            .find(|m| matches!(m.get_stable_info(), Ok((mid, _)) if mid == id))
            .ok_or("Monitor not found")?;
        monitor.set_hdr_state(state)
    }
}
