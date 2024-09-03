use std::sync::atomic::{AtomicBool, Ordering};

use seelen_core::system_state::UIColors;
use tauri::Emitter;

use crate::{log_error, seelen::get_app_handle, trace_lock};

use super::application::SYSTEM_SETTINGS;

fn emit_colors(colors: &UIColors) {
    get_app_handle()
        .emit("colors", colors)
        .expect("failed to emit");
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_colors_events() {
    let was_registered = REGISTERED.load(Ordering::Acquire);
    if !was_registered {
        REGISTERED.store(true, Ordering::Release);
    }
    std::thread::spawn(move || {
        let mut manager = trace_lock!(SYSTEM_SETTINGS);
        if !was_registered {
            log::trace!("Registering colors events");
            manager.on_colors_change(Box::new(emit_colors));
        }
        emit_colors(&manager.get_colors().expect("Failed to get colors"));
    });
}

pub fn release_colors_events() {
    if REGISTERED.load(Ordering::Acquire) {
        log_error!(trace_lock!(SYSTEM_SETTINGS).release());
    }
}
