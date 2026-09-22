use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Color, NightlightSettings, UIColors},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::system_settings::{
        application::{SystemSettings, SystemSettingsEvent},
        nightlight,
    },
};

/// Lazy initialization wrapper that registers Tauri events on first access
/// This keeps Tauri logic separate from system logic while ensuring lazy initialization
fn get_system_settings() -> &'static SystemSettings {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        SystemSettings::subscribe(|event| {
            if event == SystemSettingsEvent::ColorChanged
                && let Ok(colors) = SystemSettings::instance().get_colors()
            {
                emit_to_webviews(SeelenEvent::ColorsChanged, &colors);
            }

            if event == SystemSettingsEvent::ColorSchemeSwitched
                && let Ok(is_dark) = SystemSettings::instance().get_dark_mode()
            {
                emit_to_webviews(SeelenEvent::DarkModeChanged, &is_dark);
            }
        });
    });
    SystemSettings::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_system_colors() -> Result<UIColors> {
        get_system_settings().get_colors()
    }

    pub fn set_system_accent_color(color: Color) -> Result<()> {
        SystemSettings::set_accent_color(color)
    }

    pub fn get_system_dark_mode() -> Result<bool> {
        get_system_settings().get_dark_mode()
    }

    pub fn set_system_dark_mode(enabled: bool) -> Result<()> {
        SystemSettings::set_dark_mode(enabled)
    }

    pub fn get_system_night_light_settings() -> Result<NightlightSettings> {
        nightlight::get_settings()
    }

    pub fn get_system_night_light_enabled() -> Result<bool> {
        nightlight::get_enabled()
    }

    pub fn set_system_night_light_enabled(enabled: bool) -> Result<()> {
        nightlight::set_enabled(enabled)
    }

    pub fn set_system_night_light_color_temperature(temperature: u16) -> Result<()> {
        nightlight::set_color_temperature(temperature)
    }
}
