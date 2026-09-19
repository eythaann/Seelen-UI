use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::MonitorBrightness};

use crate::{app::emit_to_webviews, error::Result};

use super::application::{BrightnessManager, BrightnessManagerEvent};

fn get_brightness_manager() -> &'static BrightnessManager {
    static TAURI_BRIGHTNESS_REGISTRATION: Once = Once::new();
    TAURI_BRIGHTNESS_REGISTRATION.call_once(|| {
        BrightnessManager::subscribe(|event| match event {
            BrightnessManagerEvent::Changed(brightness) => {
                emit_to_webviews(SeelenEvent::SystemMonitorsBrightnessChanged, brightness);
            }
        });
    });
    BrightnessManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_all_monitors_brightness() -> Result<Vec<MonitorBrightness>> {
        Ok(get_brightness_manager().get_all_brightness())
    }

    pub fn set_monitor_brightness(instance_name: String, level: u8) -> Result<()> {
        get_brightness_manager().set_brightness(&instance_name, level)
    }
}
