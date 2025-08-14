use seelen_core::{handlers::SeelenEvent, system_state::AppNotification};
use tauri::Emitter;

use crate::{
    app::get_app_handle, error_handler::Result, log_error,
    modules::notifications::application::NotificationManager, trace_lock,
};

use super::application::NOTIFICATION_MANAGER;

pub fn register_notification_events() {
    std::thread::spawn(|| {
        log_error!(trace_lock!(NOTIFICATION_MANAGER).initialize());

        NotificationManager::subscribe(|_event| {
            log_error!(get_app_handle().emit(
                SeelenEvent::Notifications,
                trace_lock!(NOTIFICATION_MANAGER).notifications(),
            ));
        });
    });
}

pub fn release_notification_events() {
    log_error!(trace_lock!(NOTIFICATION_MANAGER).release());
}

#[tauri::command(async)]
pub fn get_notifications() -> Vec<AppNotification> {
    trace_lock!(NOTIFICATION_MANAGER).notifications().clone()
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
