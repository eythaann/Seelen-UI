//! All the work runs on a single worker thread: it owns the native layer and the saved state,
//! so requests are serialized without locks.

#[cfg(windows)]
mod windows;

use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crossbeam_channel::{Sender, bounded, unbounded};

use crate::{error::Result, platform};

use super::domain::{
    NativeWallpaperSystem, OriginalWallpaper, SavedState, black_bmp, is_seelen_owned, paths_eq,
};

#[cfg(windows)]
type Platform = windows::WindowsNativeWallpaper;

const STATE_FILENAME: &str = "state.json";
const BLACK_FILENAME: &str = "black.bmp";

pub struct NativeWallpaperManager {
    worker: Sender<Request>,
}

impl NativeWallpaperManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<NativeWallpaperManager> =
            LazyLock::new(NativeWallpaperManager::create);
        &MANAGER
    }

    fn create() -> Self {
        Self {
            worker: spawn_worker(),
        }
    }

    /// Sets the native wallpaper, `None` means black. Blocks until applied.
    pub fn set(&self, path: Option<PathBuf>) -> Result<()> {
        self.request(Job::Set(path))
    }

    /// Restores the user's original wallpaper, no-op if it was never changed. Blocks until restored.
    pub fn restore(&self) -> Result<()> {
        self.request(Job::Restore)
    }

    fn request(&self, job: Job) -> Result<()> {
        let (tx, rx) = bounded(1);
        self.worker
            .send((job, tx))
            .map_err(|_| "Native wallpaper worker is not running")?;
        rx.recv()?
    }
}

enum Job {
    Set(Option<PathBuf>),
    Restore,
}

type Request = (Job, Sender<Result<()>>);

struct Worker {
    platform: Platform,
    folder: Option<PathBuf>,
    /// `Some` while the native wallpaper is managed by us.
    state: Option<SavedState>,
}

impl Worker {
    fn load(platform: Platform) -> Self {
        let folder = platform::local_data_dir()
            .map(|dir| dir.join("native-wallpaper"))
            .inspect_err(|err| log::error!("Native wallpaper folder unavailable: {err:?}"))
            .ok();
        let state = folder.as_ref().and_then(|folder| {
            let content = std::fs::read(folder.join(STATE_FILENAME)).ok()?;
            match serde_json::from_slice::<SavedState>(&content) {
                Ok(state) => {
                    log::info!("Found the user's wallpaper saved by a previous session");
                    Some(state)
                }
                Err(err) => {
                    log::error!("Invalid saved native wallpaper state: {err:?}");
                    None
                }
            }
        });
        Self {
            platform,
            folder,
            state,
        }
    }

    fn folder(&self) -> Result<&Path> {
        Ok(self
            .folder
            .as_deref()
            .ok_or("Native wallpaper folder unavailable")?)
    }

    fn set(&mut self, path: Option<PathBuf>) -> Result<()> {
        let target = match path {
            Some(path) if path.is_file() => path,
            _ => self.black_wallpaper()?,
        };
        let target_str = target.to_string_lossy().to_string();

        let current = self.platform.wallpapers()?;

        match &mut self.state {
            None => {
                self.state = Some(SavedState {
                    original: self.platform.snapshot(current.clone())?,
                    applied: None,
                });
            }
            Some(state) => {
                // the user (or another app) changed the wallpaper while we managed it,
                // that one is now the wallpaper to restore.
                let changed_externally = current.iter().any(|(_, wallpaper)| {
                    !state.is_applied(wallpaper) && !is_seelen_owned(wallpaper)
                });
                if changed_externally {
                    log::info!(
                        "Native wallpaper was changed externally, saving it as the original"
                    );
                    state.original = self.platform.snapshot(current.clone())?;
                    state.applied = None;
                }
            }
        }

        let state = self.state.as_mut().expect("state was set above");
        let already_applied = state.is_applied(&target_str)
            && current
                .iter()
                .all(|(_, wallpaper)| paths_eq(wallpaper, &target_str));
        if already_applied {
            return Ok(());
        }

        // persisted before touching the wallpaper, so a crash never loses the original one
        state.applied = Some(target_str.clone());
        self.save()?;

        log::debug!("Setting native wallpaper: {target_str}");
        self.platform.set_filled_wallpaper(&target_str)
    }

    fn restore(&mut self) -> Result<()> {
        let Some(state) = self.state.take() else {
            return Ok(());
        };

        log::info!("Restoring the user's native wallpaper");
        if let Err(err) = self.apply_original(&state.original) {
            // keep it to try again on the next restore
            self.state = Some(state);
            return Err(err);
        }

        if let Ok(folder) = self.folder() {
            let _ = std::fs::remove_file(folder.join(STATE_FILENAME));
        }
        Ok(())
    }

    fn apply_original(&self, original: &OriginalWallpaper) -> Result<()> {
        self.platform.set_style(original)?;

        if let Some(slideshow) = &original.slideshow {
            match self.platform.set_slideshow(slideshow) {
                Ok(()) => return Ok(()),
                // e.g. its folder was deleted, the static wallpapers are restored instead
                Err(err) => log::error!("Failed to restore the slideshow: {err:?}"),
            }
        }

        let fallback = original
            .by_monitor
            .first()
            .map(|(_, wallpaper)| wallpaper.as_str())
            .unwrap_or_default();
        let same_for_all = original
            .by_monitor
            .iter()
            .all(|(_, wallpaper)| paths_eq(wallpaper, fallback));

        if same_for_all {
            return self.platform.set_wallpaper(None, fallback);
        }

        for (monitor, _) in self.platform.wallpapers()? {
            let wallpaper = original
                .by_monitor
                .iter()
                .find(|(saved, _)| saved.eq_ignore_ascii_case(&monitor))
                .map(|(_, wallpaper)| wallpaper.as_str())
                .unwrap_or(fallback);
            if let Err(err) = self.platform.set_wallpaper(Some(&monitor), wallpaper) {
                log::error!("Failed to restore the wallpaper of monitor {monitor}: {err:?}");
            }
        }
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let folder = self.folder()?;
        std::fs::create_dir_all(folder)?;
        std::fs::write(
            folder.join(STATE_FILENAME),
            serde_json::to_vec_pretty(&self.state)?,
        )?;
        Ok(())
    }

    /// Solid black image, stretched by the `Fill` position.
    fn black_wallpaper(&self) -> Result<PathBuf> {
        let folder = self.folder()?;
        let path = folder.join(BLACK_FILENAME);
        if !path.is_file() {
            std::fs::create_dir_all(folder)?;
            std::fs::write(&path, black_bmp())?;
        }
        Ok(path)
    }
}

fn spawn_worker() -> Sender<Request> {
    let (tx, rx) = unbounded::<Request>();
    let spawned = std::thread::Builder::new()
        .name("Native Wallpaper".to_string())
        .spawn(move || {
            let mut platform = Platform::create();
            if let Err(err) = platform.init() {
                log::error!("Native wallpaper worker could not be initialized: {err:?}");
            }
            let mut worker = Worker::load(platform);
            for (job, response) in rx {
                let result = match job {
                    Job::Set(path) => worker.set(path),
                    Job::Restore => worker.restore(),
                };
                let _ = response.send(result);
            }
        });
    if let Err(err) = spawned {
        log::error!("Failed to spawn the native wallpaper worker: {err:?}");
    }
    tx
}
