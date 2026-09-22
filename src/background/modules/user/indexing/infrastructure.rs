use std::{path::PathBuf, sync::Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{FolderChangedArgs, FolderType},
};

use crate::app::emit_to_webviews;

use super::application::{UserFoldersManager, UserFoldersManagerEvent};

fn register_folder_events() {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        UserFoldersManager::subscribe(|event| match event {
            UserFoldersManagerEvent::FolderChanged(folder) => {
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
}

/// Returns false and kicks off the initialization out of the caller thread when the
/// manager isn't built yet, see [`UserFoldersManager::init_in_background`].
fn ensure_folders_manager() -> bool {
    if UserFoldersManager::is_initialized() {
        return true;
    }
    register_folder_events();
    UserFoldersManager::init_in_background();
    false
}

fn get_user_folder_content(folder_type: FolderType) -> Vec<PathBuf> {
    // same as `get_user`, `SeelenEvent::UserFolderChanged` will bring the real content
    // once the background indexing finishes
    if !ensure_folders_manager() {
        return Vec::new();
    }
    let manager = UserFoldersManager::instance().lock();
    match manager.folders.get(&folder_type) {
        Some(details) => details.content.clone(),
        None => Vec::new(),
    }
}

impl crate::tauri_handlers::Handlers {
    pub fn get_user_folder_content(folder_type: FolderType) -> Vec<PathBuf> {
        get_user_folder_content(folder_type)
    }
}
