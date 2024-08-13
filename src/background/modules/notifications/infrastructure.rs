use std::sync::atomic::{AtomicBool, Ordering};

use tauri::Emitter;

use crate::{error_handler::Result, log_error, seelen::get_app_handle, trace_lock};

use super::application::{AppNotification, NOTIFICATION_MANAGER};

fn emit_notifications(notifications: &Vec<AppNotification>) {
    get_app_handle()
        .emit("notifications", notifications)
        .expect("failed to emit");
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_notification_events() {
    std::thread::spawn(|| {
        let mut manager = trace_lock!(NOTIFICATION_MANAGER);
        if !REGISTERED.load(Ordering::Acquire) {
            log::info!("Registering notifications events");
            manager.on_notifications_change(emit_notifications);
            REGISTERED.store(true, Ordering::Release);
        }
        emit_notifications(manager.notifications());
    });
}

pub fn release_notification_events() {
    if REGISTERED.load(Ordering::Acquire) {
        log_error!(trace_lock!(NOTIFICATION_MANAGER).release());
    }
}

#[tauri::command]
pub fn notifications_close(id: u32) -> Result<()> {
    trace_lock!(NOTIFICATION_MANAGER).remove_notification(id)?;
    Ok(())
}

#[tauri::command]
pub fn notifications_close_all() -> Result<()> {
    trace_lock!(NOTIFICATION_MANAGER).clear_notifications()
}
