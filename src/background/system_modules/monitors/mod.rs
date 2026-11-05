mod application;
mod domain;

use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{MonitorId, PhysicalMonitor},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    system_modules::brightness::{BrightnessEvent, BrightnessManager},
};

pub use self::{application::MonitorManager, domain::MonitorManagerEvent};

/// Manager instance, bridging its events (and the brightness ones) to the webviews on first
/// access.
fn get_manager() -> &'static MonitorManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        log::debug!(
            "Initial monitors: {:#?}",
            MonitorManager::instance().get_cached_data()
        );

        MonitorManager::subscribe(|event| {
            log::trace!("MonitorManagerEvent::{event:?}");
            let monitors = monitors();
            log::debug!("{monitors:#?}");
            emit_to_webviews(SeelenEvent::SystemMonitorsChanged, monitors);
        });

        BrightnessManager::subscribe(|event| match event {
            BrightnessEvent::Changed => {
                emit_to_webviews(SeelenEvent::SystemMonitorsChanged, monitors());
            }
        });
    });
    MonitorManager::instance()
}

/// The connected monitors together with their brightness, which is tracked apart from the
/// display topology by `BrightnessManager`.
fn monitors() -> Vec<PhysicalMonitor> {
    let brightness = BrightnessManager::instance();
    let mut monitors = MonitorManager::instance().get_cached_data();
    for monitor in &mut monitors {
        monitor.brightness = brightness.brightness(&monitor.id);
    }
    monitors
}

impl crate::tauri_handlers::Handlers {
    pub fn get_connected_monitors() -> Vec<PhysicalMonitor> {
        get_manager();
        monitors()
    }

    pub fn set_monitor_hdr(id: MonitorId, state: bool) -> Result<()> {
        get_manager().set_hdr(&id, state)
    }

    pub fn set_monitor_brightness(id: MonitorId, brightness: u8) -> Result<()> {
        get_manager();
        BrightnessManager::instance().set_brightness(&id, brightness)
    }
}
