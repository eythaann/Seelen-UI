use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::MonitorBrightness};

use crate::{app::emit_to_webviews, error::Result};

use super::application::{BrightnessManager, BrightnessManagerEvent};

fn get_brightness_manager() -> &'static BrightnessManager {
    static TAURI_BRIGHTNESS_REGISTRATION: Once = Once::new();
    TAURI_BRIGHTNESS_REGISTRATION.call_once(|| {
        BrightnessManager::subscribe(|event| match event {
            BrightnessManagerEvent::Changed(brightness) => {
                let brightness_data: Vec<MonitorBrightness> = brightness
                    .into_iter()
                    .map(|b| MonitorBrightness {
                        instance_name: b.instance_name,
                        current_brightness: b.current_brightness,
                        levels: b.levels,
                        available_levels: b.level,
                        active: b.active,
                    })
                    .collect();

                emit_to_webviews(
                    SeelenEvent::SystemMonitorsBrightnessChanged,
                    brightness_data,
                );
            }
        });
    });
    BrightnessManager::instance()
}

#[tauri::command(async)]
pub fn get_all_monitors_brightness() -> Result<Vec<MonitorBrightness>> {
    let manager = get_brightness_manager();
    let brightness = manager.get_all_brightness();

    Ok(brightness
        .into_iter()
        .map(|b| MonitorBrightness {
            instance_name: b.instance_name,
            current_brightness: b.current_brightness,
            levels: b.levels,
            available_levels: b.level,
            active: b.active,
        })
        .collect())
}

#[tauri::command(async)]
pub fn set_monitor_brightness(instance_name: String, level: u8) -> Result<()> {
    let manager = get_brightness_manager();
    manager.set_brightness(&instance_name, level)
}
