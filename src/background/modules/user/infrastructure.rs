use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{
    error_handler::AppError,
    log_error,
    modules::user::{UserManagerEvent, USER_MANAGER},
    seelen::get_app_handle,
    trace_lock,
};

use super::application::UserManager;

use seelen_core::system_state::{File, FolderChangedArgs, FolderType, User};

fn _get_user() -> Result<User, AppError> {
    let user = { trace_lock!(USER_MANAGER).user_details().clone().unwrap() };
    Ok(user)
}

pub fn register_user_events() {
    log::trace!("Register for user profile events!");
    UserManager::subscribe(|event| match event {
        UserManagerEvent::UserUpdated() => {
            if let Ok(user) = _get_user() {
                log_error!(get_app_handle().emit(SeelenEvent::UserChanged, user));
            }
        }
        UserManagerEvent::FolderChanged(folder) => {
            log_error!(get_app_handle().emit(
                SeelenEvent::UserFolderChanged,
                FolderChangedArgs {
                    of_folder: folder.clone(),
                    content: get_user_folder_content(folder).ok(),
                }
            ));
        }
    });
}

#[tauri::command(async)]
pub fn get_user() -> Result<User, AppError> {
    _get_user()
}

#[tauri::command(async)]
pub fn get_user_folder_content(folder_type: FolderType) -> Result<Vec<File>, AppError> {
    let manager = trace_lock!(USER_MANAGER);

    let result = manager.folders()[&folder_type]
        .content()
        .as_ref()
        .unwrap()
        .to_vec();

    Ok(result)
}

#[tauri::command(async)]
pub fn set_user_folder_limit(folder_type: FolderType, amount: usize) -> Result<(), AppError> {
    let mut manager = trace_lock!(USER_MANAGER);
    manager.set_folder_limit(folder_type, amount)?;
    Ok(())
}
