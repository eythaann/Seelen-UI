mod application;
mod domain;

use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Color, NightlightSettings, SystemSettingEntry, UIColors},
};

use crate::{app::emit_to_webviews, error::Result};

pub use self::{application::SystemSettings, domain::SystemSettingsEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static SystemSettings {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        SystemSettings::subscribe(|event| match event {
            SystemSettingsEvent::ColorChanged => {
                if let Ok(colors) = SystemSettings::instance().get_colors() {
                    emit_to_webviews(SeelenEvent::ColorsChanged, &colors);
                }
            }
            SystemSettingsEvent::ColorSchemeSwitched => {
                if let Ok(is_dark) = SystemSettings::instance().get_dark_mode() {
                    emit_to_webviews(SeelenEvent::DarkModeChanged, is_dark);
                }
            }
            SystemSettingsEvent::TextScaleChanged => {}
        });
    });
    SystemSettings::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_system_colors() -> Result<UIColors> {
        get_manager().get_colors()
    }

    pub fn set_system_accent_color(color: Color) -> Result<()> {
        get_manager().set_accent_color(color)
    }

    pub fn get_system_dark_mode() -> Result<bool> {
        get_manager().get_dark_mode()
    }

    pub fn set_system_dark_mode(enabled: bool) -> Result<()> {
        get_manager().set_dark_mode(enabled)
    }

    pub fn enum_system_settings() -> Vec<SystemSettingEntry> {
        get_manager().settings_entries()
    }

    pub fn get_system_night_light_settings() -> Result<NightlightSettings> {
        get_manager().night_light_settings()
    }

    pub fn get_system_night_light_enabled() -> Result<bool> {
        get_manager().is_night_light_enabled()
    }

    pub fn set_system_night_light_enabled(enabled: bool) -> Result<()> {
        get_manager().set_night_light_enabled(enabled)
    }

    pub fn set_system_night_light_color_temperature(temperature: u16) -> Result<()> {
        get_manager().set_night_light_temperature(temperature)
    }
}
