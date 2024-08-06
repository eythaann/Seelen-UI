use std::sync::atomic::{AtomicBool, Ordering};

use tauri::Emitter;

use crate::{log_error, seelen::get_app_handle};

use super::{application::SYSTEM_SETTINGS, domain::UIColors};

fn emit_colors(colors: &UIColors) {
    get_app_handle()
        .emit("colors", colors)
        .expect("failed to emit");
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_colors_events() {
    std::thread::spawn(|| {
        let mut manager = SYSTEM_SETTINGS.lock();
        if !REGISTERED.load(Ordering::Acquire) {
            manager.on_colors_change(Box::new(emit_colors));
            REGISTERED.store(true, Ordering::Release);
        }
        emit_colors(&manager.get_colors().expect("Failed to get colors"));
    });
}

pub fn release_colors_events() {
    if REGISTERED.load(Ordering::Acquire) {
        log_error!(SYSTEM_SETTINGS.lock().release());
    }
}
