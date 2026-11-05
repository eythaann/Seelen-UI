use std::{path::PathBuf, sync::Arc, time::Duration};

use notify_debouncer_full::{
    DebounceEventResult, Debouncer, FileIdMap, new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
};
use seelen_core::system_state::FolderType;
use tauri::Manager;

use crate::{
    app::get_app_handle,
    error::Result,
    system_modules::user_folders::domain::{UserFoldersEvent, UserFoldersSystem},
};

const WATCHER_DEBOUNCE: Duration = Duration::from_secs(1);

pub struct WindowsUserFolders {
    /// One per watched folder, dropping them stops the watching.
    watchers: Vec<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>,
}

impl UserFoldersSystem for WindowsUserFolders {
    fn create() -> Self {
        Self {
            watchers: Vec::new(),
        }
    }

    fn init<F>(&mut self, folders: &[(FolderType, PathBuf)], on_event: F) -> Result<()>
    where
        F: Fn(UserFoldersEvent) + Send + Sync + 'static,
    {
        let on_event = Arc::new(on_event);
        for (folder, path) in folders {
            match watch_folder(*folder, path, on_event.clone()) {
                Ok(watcher) => self.watchers.push(watcher),
                Err(err) => {
                    log::error!("Failed to watch user folder {folder:?} ({path:?}): {err:?}")
                }
            }
        }
        Ok(())
    }

    fn folder_path(&self, folder: FolderType) -> Option<PathBuf> {
        let resolver = get_app_handle().path();
        match folder {
            FolderType::Recent => {
                Some(resolver.data_dir().ok()?.join("Microsoft\\Windows\\Recent"))
            }
            FolderType::Desktop => resolver.desktop_dir().ok(),
            FolderType::Downloads => resolver.download_dir().ok(),
            FolderType::Documents => resolver.document_dir().ok(),
            FolderType::Pictures => resolver.picture_dir().ok(),
            FolderType::Videos => resolver.video_dir().ok(),
            FolderType::Music => resolver.audio_dir().ok(),
        }
    }
}

fn watch_folder<F>(
    folder: FolderType,
    path: &PathBuf,
    on_event: Arc<F>,
) -> Result<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>
where
    F: Fn(UserFoldersEvent) + Send + Sync + 'static,
{
    let mut debouncer = new_debouncer(
        WATCHER_DEBOUNCE,
        None,
        move |result: DebounceEventResult| match result {
            Ok(_events) => on_event(UserFoldersEvent::FolderChanged { folder }),
            Err(errors) => log::error!("Folder Watcher Error for {folder:?}: {errors:?}"),
        },
    )?;
    debouncer.watch(path, RecursiveMode::Recursive)?;
    Ok(debouncer)
}
