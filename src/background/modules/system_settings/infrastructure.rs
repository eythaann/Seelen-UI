use seelen_core::{handlers::SeelenEvent, system_state::UIColors};
use tauri::Emitter;

use crate::{
    app::get_app_handle,
    error_handler::Result,
    log_error,
    modules::system_settings::application::{SystemSettings, SystemSettingsEvent},
    trace_lock,
};

use super::application::SYSTEM_SETTINGS;

fn emit_colors(colors: &UIColors) {
    get_app_handle()
        .emit(SeelenEvent::ColorsChanged, colors)
        .expect("failed to emit");
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
