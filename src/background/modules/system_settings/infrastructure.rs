use seelen_core::{handlers::SeelenEvent, system_state::UIColors};
use tauri::Emitter;

use crate::{error_handler::Result, log_error, seelen::get_app_handle, trace_lock};

use super::application::SYSTEM_SETTINGS;

fn emit_colors(colors: &UIColors) {
    get_app_handle()
        .emit(SeelenEvent::ColorsChanged, colors)
        .expect("failed to emit");
}

pub fn register_colors_events() {
    std::thread::spawn(move || {
        let mut manager = trace_lock!(SYSTEM_SETTINGS);
        manager.on_colors_change(Box::new(emit_colors));
    });
}

pub fn release_colors_events() {
    log_error!(trace_lock!(SYSTEM_SETTINGS).release());
}

#[tauri::command(async)]
pub fn get_system_colors() -> Result<UIColors> {
    trace_lock!(SYSTEM_SETTINGS).get_colors()
}
