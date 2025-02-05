use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{
    error_handler::Result,
    log_error,
    modules::user::{UserManagerEvent, USER_MANAGER},
    seelen::get_app_handle,
    trace_lock,
};

use super::application::UserManager;

use seelen_core::system_state::{File, FolderChangedArgs, FolderType, User};

pub fn register_user_events() {
    log::trace!("Register for user profile events!");
    UserManager::subscribe(|event| match event {
        UserManagerEvent::UserUpdated() => {
            let guard = trace_lock!(USER_MANAGER);
            log_error!(get_app_handle().emit(SeelenEvent::UserChanged, &guard.user));
        }
        UserManagerEvent::FolderChanged(folder) => {
            log_error!(get_app_handle().emit(
                SeelenEvent::UserFolderChanged,
                FolderChangedArgs {
                    of_folder: folder,
                    content: get_user_folder_content(folder).ok(),
                }
            ));
        }
    });
}

#[tauri::command(async)]
pub fn get_user() -> User {
    trace_lock!(USER_MANAGER).user.clone()
}

#[tauri::command(async)]
pub fn get_user_folder_content(folder_type: FolderType) -> Result<Vec<File>> {
    let manager = trace_lock!(USER_MANAGER);
    match manager.folders.get(&folder_type) {
        Some(details) => Ok(details.content.clone()),
        None => Err("Invalid folder type".into()),
    }
}

#[tauri::command(async)]
pub fn set_user_folder_limit(folder_type: FolderType, amount: usize) -> Result<()> {
    let mut manager = trace_lock!(USER_MANAGER);
    manager.set_folder_limit(folder_type, amount)?;
    Ok(())
}
