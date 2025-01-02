use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{
    error_handler::AppError, log_error, modules::user::USER_MANAGER, seelen::get_app_handle,
    trace_lock,
};

use super::{
    application::UserManager,
    domain::{ExposedRecentFile, User},
};

fn _get_user() -> Result<User, AppError> {
    let user = { trace_lock!(USER_MANAGER).user_details().clone().unwrap() };
    Ok(user)
}

pub fn register_user_events() {
    //Initialize the User Manager for first use.
    log::trace!("Register for user profile events!");
    _ = _get_user();

    UserManager::subscribe(|event| match event {
        crate::modules::user::UserManagerEvent::UserUpdated() => {
            if let Ok(user) = _get_user() {
                log_error!(get_app_handle().emit(SeelenEvent::UserChanged, user));
            }
        }
        crate::modules::user::UserManagerEvent::RecentFolderChanged() => {
            log_error!(get_app_handle().emit(
                SeelenEvent::UserRecentFolderChanged,
                get_user_recent_folder_content().ok().unwrap()
            ));
        }
    });
}

#[tauri::command(async)]
pub fn get_user() -> Result<User, AppError> {
    _get_user()
}

#[tauri::command(async)]
pub fn get_user_recent_folder_content() -> Result<Vec<ExposedRecentFile>, AppError> {
    let manager = trace_lock!(USER_MANAGER);

    let result = manager
        .recent_folder()
        .as_ref()
        .unwrap()
        .iter()
        .take(*manager.recent_folder_limit())
        .map(|item| item.clone().into())
        .collect();

    Ok(result)
}

#[tauri::command(async)]
pub fn set_user_recent_folder_limit(amount: usize) -> Result<(), AppError> {
    let mut manager = trace_lock!(USER_MANAGER);
    manager.set_recent_folder_limit(amount)?;
    Ok(())
}
