#[cfg(windows)]
mod windows;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        LazyLock, Once,
        atomic::{AtomicBool, Ordering},
    },
};

use seelen_core::system_state::FolderType;

use crate::{error::ResultLogExt, event_manager, utils::lock_free::TracedMutex};

use super::domain::{
    UserFoldersEvent, UserFoldersSystem, is_ignored_entry, is_ignored_file, max_scan_depth,
};

#[cfg(windows)]
type Platform = windows::WindowsUserFolders;

/// Set once the first (potentially slow, huge/deep trees or slow/unavailable paths)
/// indexing of the known user folders has finished.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct UserFoldersManager {
    platform: Platform,
    folders: TracedMutex<HashMap<FolderType, FolderDetails>>,
}

event_manager!(UserFoldersManager, UserFoldersEvent);

impl UserFoldersManager {
    /// Blocks until the folders are indexed, prefer [`Self::is_initialized`] + this, or
    /// [`Self::init_in_background`], on paths that shouldn't wait (like IPC commands).
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<UserFoldersManager> = LazyLock::new(|| {
            let mut m = UserFoldersManager::create();
            m.init();
            INITIALIZED.store(true, Ordering::Release);
            m
        });
        &MANAGER
    }

    /// Indexing the known user folders scans and starts watching all of them, which on
    /// some systems takes a long time (huge trees, slow/unavailable paths). Doing that
    /// lazily from an IPC command left widgets waiting forever on `get_user_folder_content`,
    /// so the work is done once on a background thread instead.
    pub fn init_in_background() {
        static SPAWNED: Once = Once::new();
        SPAWNED.call_once(|| {
            std::thread::spawn(|| {
                let _ = Self::instance();
                // widgets that got an empty list while this was running are updated by these
                for &folder in FolderType::values() {
                    Self::send(UserFoldersEvent::FolderChanged { folder });
                }
            });
        });
    }

    pub fn is_initialized() -> bool {
        INITIALIZED.load(Ordering::Acquire)
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            folders: TracedMutex::new(HashMap::new()),
        }
    }

    /// Infallible on purpose: folders that can't be read or watched are skipped or degraded,
    /// so a broken folder never leaves the others unindexed.
    fn init(&mut self) {
        let mut found = Vec::new();
        for &folder in FolderType::values() {
            let Some(path) = self.platform.folder_path(folder) else {
                continue;
            };
            if !path.is_dir() {
                log::warn!("Skipping user folder {folder:?}, missing directory: {path:?}");
                continue;
            }
            found.push((folder, path));
        }

        let mut folders = HashMap::new();
        for (folder, path) in &found {
            let content = scan_folder(path, *folder);
            folders.insert(
                *folder,
                FolderDetails {
                    path: path.clone(),
                    content,
                },
            );
        }
        *self.folders.lock() = folders;

        self.platform
            .init(&found, Self::on_platform_event)
            .log_error();
    }

    fn on_platform_event(event: UserFoldersEvent) {
        match event {
            UserFoldersEvent::FolderChanged { folder } => {
                if Self::instance().reload_folder(folder) {
                    Self::send(event);
                }
            }
        }
    }

    /// Returns false if the folder is not indexed.
    fn reload_folder(&self, folder: FolderType) -> bool {
        let Some(path) = self.folders.lock().get(&folder).map(|d| d.path.clone()) else {
            return false;
        };
        // scanning can be slow, so it is done without holding the lock
        let content = scan_folder(&path, folder);
        if let Some(details) = self.folders.lock().get_mut(&folder) {
            details.content = content;
        }
        true
    }

    pub fn folder_content(&self, folder: FolderType) -> Vec<PathBuf> {
        self.folders
            .lock()
            .get(&folder)
            .map(|details| details.content.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug)]
struct FolderDetails {
    path: PathBuf,
    content: Vec<PathBuf>,
}

/// Files of the folder, skipping the ignored entries.
fn scan_folder(base_path: &Path, folder: FolderType) -> Vec<PathBuf> {
    walkdir::WalkDir::new(base_path)
        .follow_links(false)
        .max_depth(max_scan_depth(folder))
        .into_iter()
        .filter_entry(|e| !is_ignored_entry(e.path()))
        .filter_map(|e| e.ok())
        .map(|entry| entry.into_path())
        .filter(|path| path.is_file() && !is_ignored_file(path))
        .collect()
}
