use seelen_core::{handlers::SeelenEvent, system_state::ApplicationHistoryEntry};
use tauri::Emitter;

use crate::{
    error_handler::AppError,
    log_error,
    modules::application_history::{
        ApplicationHistory, ApplicationHistoryEvent, APPLICATION_HISTORY,
    },
    seelen::get_app_handle,
    seelen_bar::FancyToolbar,
    trace_lock,
    windows_api::window::Window,
};

fn _get_history() -> Vec<ApplicationHistoryEntry> {
    let history = trace_lock!(APPLICATION_HISTORY);

    history.history().to_vec()
}

pub fn register_application_history_events() {
    log::trace!("Register for application history!");
    ApplicationHistory::subscribe(|event| match event {
        ApplicationHistoryEvent::ApplicationHistoryAdded(focused_app) => {
            log_error!(get_app_handle().emit(SeelenEvent::GlobalFocusChanged, focused_app,));
        }
        ApplicationHistoryEvent::CurrentItemModified(focused_app) => {
            log_error!(get_app_handle().emit(SeelenEvent::GlobalFocusChanged, focused_app,));
        }
        ApplicationHistoryEvent::ApplicationHistoryChanged => {
            log_error!(get_app_handle().emit(SeelenEvent::GlobalHistoryChanged, _get_history(),));
        }
        ApplicationHistoryEvent::ApplicationHistoryByMonitorChanged(monitor_id, items) => {
            log_error!(get_app_handle().emit_to(
                FancyToolbar::get_label(&monitor_id),
                SeelenEvent::HistoryChangedOnMonitor,
                items,
            ));
        }
    });
}

#[tauri::command(async)]
pub fn get_application_history() -> Result<Vec<ApplicationHistoryEntry>, AppError> {
    Ok(_get_history())
}

#[tauri::command(async)]
pub fn get_application_history_by_monitor(
    window: tauri::Window,
) -> Result<Vec<ApplicationHistoryEntry>, AppError> {
    let device_id = Window::from(window.hwnd()?).monitor().device_id()?;
    let items = trace_lock!(APPLICATION_HISTORY).get_filtered_by_monitor()?;

    Ok(items[&device_id].clone())
}

#[tauri::command(async)]
pub fn set_application_history_limit(amount: usize) -> Result<(), AppError> {
    let mut history = trace_lock!(APPLICATION_HISTORY);
    history.set_limit(amount)?;
    Ok(())
}
