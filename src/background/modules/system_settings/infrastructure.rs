use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Color, UIColors},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::system_settings::application::{SystemSettings, SystemSettingsEvent},
};

/// Lazy initialization wrapper that registers Tauri events on first access
/// This keeps Tauri logic separate from system logic while ensuring lazy initialization
fn get_system_settings() -> &'static SystemSettings {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        SystemSettings::subscribe(|event| {
            if event == SystemSettingsEvent::ColorChanged {
                if let Ok(colors) = SystemSettings::instance().get_colors() {
                    emit_to_webviews(SeelenEvent::ColorsChanged, &colors);
                }
            }
        });
    });
    SystemSettings::instance()
}

#[tauri::command(async)]
pub fn get_system_colors() -> Result<UIColors> {
    get_system_settings().get_colors()
}

#[tauri::command(async)]
pub fn set_system_accent_color(color: Color) -> Result<()> {
    SystemSettings::set_accent_color(color)
}
