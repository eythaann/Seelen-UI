use std::sync::atomic::{AtomicBool, Ordering};

use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{error_handler::Result, log_error, seelen::get_app_handle, trace_lock};

use super::application::{AppNotification, NOTIFICATION_MANAGER};

fn emit_notifications(notifications: &Vec<AppNotification>) {
    get_app_handle()
        .emit(SeelenEvent::Notifications, notifications)
        .expect("failed to emit");
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_notification_events() {
    let was_registered = REGISTERED.load(Ordering::Acquire);
    if !was_registered {
        REGISTERED.store(true, Ordering::Release);
    }
    std::thread::spawn(move || {
        let mut manager = trace_lock!(NOTIFICATION_MANAGER);
        if !was_registered {
            log::trace!("Registering notifications events");
            manager.on_notifications_change(emit_notifications);
        }
        emit_notifications(manager.notifications());
    });
}

pub fn release_notification_events() {
    if REGISTERED.load(Ordering::Acquire) {
        log_error!(trace_lock!(NOTIFICATION_MANAGER).release());
    }
}

#[tauri::command(async)]
pub fn notifications_close(id: u32) -> Result<()> {
    trace_lock!(NOTIFICATION_MANAGER).remove_notification(id)?;
    Ok(())
}

#[tauri::command(async)]
pub fn notifications_close_all() -> Result<()> {
    trace_lock!(NOTIFICATION_MANAGER).clear_notifications()
}
