use seelen_core::{handlers::SeelenEvent, system_state::UIColors};

use crate::{
    app::emit_to_webviews,
    error::Result,
    log_error,
    modules::system_settings::application::{SystemSettings, SystemSettingsEvent},
    trace_lock,
};

use super::application::SYSTEM_SETTINGS;

fn emit_colors(colors: &UIColors) {
    emit_to_webviews(SeelenEvent::ColorsChanged, colors);
}

pub fn register_system_settings_events() {
    std::thread::spawn(move || {
        log_error!(trace_lock!(SYSTEM_SETTINGS).initialize());
        SystemSettings::subscribe(|event| {
            if event == SystemSettingsEvent::ColorChanged {
                if let Ok(colors) = trace_lock!(SYSTEM_SETTINGS).get_colors() {
                    emit_colors(&colors);
                }
            }
        });
    });
}

pub fn release_colors_events() {
    log_error!(trace_lock!(SYSTEM_SETTINGS).release());
}

#[tauri::command(async)]
pub fn get_system_colors() -> Result<UIColors> {
    trace_lock!(SYSTEM_SETTINGS).get_colors()
}
