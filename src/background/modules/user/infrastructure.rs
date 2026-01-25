use std::{
    path::PathBuf,
    sync::{Arc, Once},
};

use parking_lot::Mutex;
use seelen_core::{
    handlers::SeelenEvent,
    system_state::{FolderChangedArgs, FolderType, User},
};

use crate::{app::emit_to_webviews, trace_lock};

use super::application::{UserManager, UserManagerEvent};

fn get_user_manager() -> &'static Arc<Mutex<UserManager>> {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        UserManager::subscribe(|event| match event {
            UserManagerEvent::UserUpdated => {
                let guard = trace_lock!(UserManager::instance());
                emit_to_webviews(SeelenEvent::UserChanged, &guard.user);
            }
            UserManagerEvent::FolderChanged(folder) => {
                emit_to_webviews(
                    SeelenEvent::UserFolderChanged,
                    FolderChangedArgs {
                        of_folder: folder,
                        content: get_user_folder_content(folder),
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
pub fn get_user_folder_content(folder_type: FolderType) -> Vec<PathBuf> {
    let manager = trace_lock!(get_user_manager());
    match manager.folders.get(&folder_type) {
        Some(details) => details.content.clone(),
        None => Vec::new(),
    }
}
