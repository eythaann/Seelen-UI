#[cfg(windows)]
mod windows;

use std::{path::Path, sync::LazyLock, time::SystemTime};

use seelen_core::state::NativeWallpaperSnapshot;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::{lock_free::TracedMutex, spawn_named_thread},
};

use super::domain::{ShellWallpaperEvent, ShellWallpaperSystem};

#[cfg(windows)]
type Platform = windows::WindowsShellWallpaper;

/// Tracks the static wallpaper currently selected in the OS. Changes are pushed by the platform
/// (setting changes and a watcher on the wallpaper folder), there is no polling.
pub struct ShellWallpaperManager {
    platform: Platform,
    snapshot: TracedMutex<NativeWallpaperSnapshot>,
}

event_manager!(ShellWallpaperManager, ShellWallpaperEvent);

impl ShellWallpaperManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<ShellWallpaperManager> = LazyLock::new(|| {
            let mut m = ShellWallpaperManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            snapshot: TracedMutex::new(NativeWallpaperSnapshot::default()),
        }
    }

    fn init(&mut self) -> Result<()> {
        let (refresh_tx, refresh_rx) = crossbeam_channel::unbounded::<()>();
        // The OS layer never knows about the manager, it only requests a refresh.
        self.platform.init(move |_event| {
            let _ = refresh_tx.send(());
        })?;

        // Initial state is read synchronously so the first command already returns real data.
        let snapshot = self.read_snapshot();
        self.watch_folder_of(&snapshot);
        *self.snapshot.lock() = snapshot;

        // A single worker serializes the refreshes (and watcher swaps), coalescing bursts.
        spawn_named_thread("Shell Wallpaper", move || {
            while refresh_rx.recv().is_ok() {
                while refresh_rx.try_recv().is_ok() {}
                Self::instance().refresh();
            }
        });
        Ok(())
    }

    pub fn snapshot(&self) -> NativeWallpaperSnapshot {
        self.snapshot.lock().clone()
    }

    fn refresh(&self) {
        let next = self.read_snapshot();
        self.watch_folder_of(&next);

        let changed = {
            let mut current = self.snapshot.lock();
            let changed = *current != next;
            if changed {
                *current = next.clone();
            }
            changed
        };

        if changed {
            log::trace!("Native wallpaper changed: {:?}", next.path);
            Self::send(ShellWallpaperEvent::Changed { snapshot: next });
        }
    }

    fn read_snapshot(&self) -> NativeWallpaperSnapshot {
        match self.platform.wallpaper() {
            Ok(path) => {
                let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
                snapshot_of(path, modified)
            }
            Err(err) => {
                log::warn!("Failed to read the native wallpaper path: {err}");
                NativeWallpaperSnapshot::default()
            }
        }
    }

    fn watch_folder_of(&self, snapshot: &NativeWallpaperSnapshot) {
        self.platform
            .watch_folder(wallpaper_folder(&snapshot.path))
            .log_error();
    }
}

fn snapshot_of(path: std::path::PathBuf, modified: Option<SystemTime>) -> NativeWallpaperSnapshot {
    if path.as_os_str().is_empty() {
        return NativeWallpaperSnapshot::default();
    }
    let revision = modified
        .and_then(|modified| modified.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default();
    NativeWallpaperSnapshot { path, revision }
}

fn wallpaper_folder(wallpaper: &Path) -> Option<&Path> {
    wallpaper
        .parent()
        .filter(|folder| !folder.as_os_str().is_empty())
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use super::*;

    #[test]
    fn empty_path_means_no_wallpaper() {
        let snapshot = snapshot_of(PathBuf::new(), Some(SystemTime::now()));
        assert_eq!(snapshot, NativeWallpaperSnapshot::default());
        assert_eq!(wallpaper_folder(&snapshot.path), None);
    }

    #[test]
    fn revision_follows_modification_time() {
        let path = PathBuf::from(r"C:\Wallpapers\TranscodedWallpaper");
        let modified = SystemTime::UNIX_EPOCH + Duration::from_millis(1234);
        let snapshot = snapshot_of(path.clone(), Some(modified));
        assert_eq!(snapshot.revision, 1234);
        assert_eq!(snapshot_of(path, None).revision, 0);
    }

    #[test]
    fn watches_the_folder_of_the_wallpaper() {
        let path = PathBuf::from(r"C:\Wallpapers\image.jpg");
        assert_eq!(wallpaper_folder(&path), Some(Path::new(r"C:\Wallpapers")));
        assert_eq!(wallpaper_folder(Path::new("image.jpg")), None);
    }
}
