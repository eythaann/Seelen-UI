mod application;
mod domain;

use std::{path::PathBuf, sync::Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{FolderChangedArgs, FolderType},
};

use crate::app::emit_to_webviews;

use self::{application::UserFoldersManager, domain::UserFoldersEvent};

/// Manager instance once the folders are indexed. Before that, returns `None` and starts the
/// indexing out of the caller thread (see [`UserFoldersManager::init_in_background`]).
fn get_manager() -> Option<&'static UserFoldersManager> {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        UserFoldersManager::subscribe(|event| match event {
            UserFoldersEvent::FolderChanged { folder } => {
                emit_to_webviews(
                    SeelenEvent::UserFolderChanged,
                    FolderChangedArgs {
                        of_folder: folder,
                        content: folder_content(folder),
                    },
                );
            }
        });
    });
    if UserFoldersManager::is_initialized() {
        return Some(UserFoldersManager::instance());
    }
    UserFoldersManager::init_in_background();
    None
}

/// Empty until the background indexing finishes; `SeelenEvent::UserFolderChanged` brings
/// the real content then.
fn folder_content(folder: FolderType) -> Vec<PathBuf> {
    get_manager()
        .map(|manager| manager.folder_content(folder))
        .unwrap_or_default()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_user_folder_content(folder_type: FolderType) -> Vec<PathBuf> {
        folder_content(folder_type)
    }
}
