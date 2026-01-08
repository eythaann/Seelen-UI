use std::sync::{Arc, Once};

use parking_lot::Mutex;
use seelen_core::{
    handlers::SeelenEvent,
    system_state::{File, FolderChangedArgs, FolderType, User},
};

use crate::{app::emit_to_webviews, error::Result, trace_lock};

use super::application::{UserManager, UserManagerEvent};

fn get_user_manager() -> &'static Arc<Mutex<UserManager>> {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        UserManager::subscribe(|event| match event {
            UserManagerEvent::UserUpdated() => {
                let guard = trace_lock!(UserManager::instance());
                emit_to_webviews(SeelenEvent::UserChanged, &guard.user);
            }
            UserManagerEvent::FolderChanged(folder) => {
                emit_to_webviews(
                    SeelenEvent::UserFolderChanged,
                    FolderChangedArgs {
                        of_folder: folder,
                        content: Some(get_user_folder_content(folder)),
                    },
                );
            }
        });
    });
    UserManager::instance()
}

#[tauri::command(async)]
pub fn get_user() -> User {
    trace_lock!(get_user_manager()).user.clone()
}

#[tauri::command(async)]
pub fn get_user_folder_content(folder_type: FolderType) -> Vec<File> {
    let manager = trace_lock!(get_user_manager());
    match manager.folders.get(&folder_type) {
        Some(details) => details.content.clone(),
        None => Vec::new(),
    }
}

#[tauri::command(async)]
pub fn set_user_folder_limit(folder_type: FolderType, amount: usize) -> Result<()> {
    let mut manager = trace_lock!(get_user_manager());
    manager.set_folder_limit(folder_type, amount)?;
    Ok(())
}
