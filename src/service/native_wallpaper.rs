//! Native desktop wallpaper mirroring.
//!
//! While Seelen UI renders its own wallpaper, the native one is replaced by a static version of it
//! (the image itself or a thumbnail, black otherwise), so system surfaces that show the native
//! wallpaper (UAC prompt, session transitions, explorer restarts, etc.) match what the user sees.
//! The user's original wallpaper is saved to disk before the first change and restored when the
//! app asks for it or when the service stops, so it also survives an app/service crash.
//! A slideshow is restored as such. Windows Spotlight has no public API, so it is restored as its
//! last image.
//!
//! All the work runs on a single worker thread: it owns the COM apartment and the saved state,
//! so requests are serialized without locks.

use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crossbeam_channel::{Sender, bounded, unbounded};
use serde::{Deserialize, Serialize};
use windows::{
    Win32::{
        Foundation::COLORREF,
        System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree},
        UI::Shell::{
            Common::ITEMIDLIST, DESKTOP_SLIDESHOW_OPTIONS, DESKTOP_WALLPAPER_POSITION,
            DSO_SHUFFLEIMAGES, DSS_SLIDESHOW, DWPOS_FILL, DesktopWallpaper, FOLDERID_LocalAppData,
            IDesktopWallpaper, ILFree, SHCreateShellItemArrayFromIDLists, SHParseDisplayName,
            SIGDN_FILESYSPATH,
        },
    },
    core::{PCWSTR, PWSTR},
};

use crate::{
    error::Result, string_utils::WindowsString, windows_api::WindowsApi, windows_api::com::Com,
};

const STATE_FILENAME: &str = "state.json";
const BLACK_FILENAME: &str = "black.bmp";

static WORKER: LazyLock<Sender<Request>> = LazyLock::new(spawn_worker);

/// Sets the native wallpaper, `None` means black. Blocks until applied.
pub fn set(path: Option<PathBuf>) -> Result<()> {
    request(Job::Set(path))
}

/// Restores the user's original wallpaper, no-op if it was never changed. Blocks until restored.
pub fn restore() -> Result<()> {
    request(Job::Restore)
}

enum Job {
    Set(Option<PathBuf>),
    Restore,
}

type Request = (Job, Sender<Result<()>>);

fn request(job: Job) -> Result<()> {
    let (tx, rx) = bounded(1);
    WORKER
        .send((job, tx))
        .map_err(|_| "Native wallpaper worker is not running")?;
    rx.recv()?
}

fn spawn_worker() -> Sender<Request> {
    let (tx, rx) = unbounded::<Request>();
    let spawned = std::thread::Builder::new()
        .name("Native Wallpaper".to_string())
        .spawn(move || {
            // S_FALSE (already initialized) is fine, it is our own thread.
            if unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) }.is_err() {
                log::error!("Native wallpaper worker could not initialize COM");
            }
            let mut worker = Worker::load();
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

struct Worker {
    folder: Option<PathBuf>,
    /// `Some` while the native wallpaper is managed by us.
    state: Option<SavedState>,
}

impl Worker {
    fn load() -> Self {
        let folder = WindowsApi::known_folder(FOLDERID_LocalAppData)
            .map(|dir| dir.join("com.seelen.seelen-ui/native-wallpaper"))
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
        Self { folder, state }
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

        let api = NativeWallpaperApi::new()?;
        let current = api.wallpapers();

        match &mut self.state {
            None => {
                self.state = Some(SavedState {
                    original: api.snapshot(current.clone())?,
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
                    state.original = api.snapshot(current.clone())?;
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
        api.set_position(DWPOS_FILL)?;
        api.set_wallpaper(None, &target_str)?;
        Ok(())
    }

    fn restore(&mut self) -> Result<()> {
        let Some(state) = self.state.take() else {
            return Ok(());
        };

        log::info!("Restoring the user's native wallpaper");
        if let Err(err) = Self::apply_original(&state.original) {
            // keep it to try again on the next restore
            self.state = Some(state);
            return Err(err);
        }

        if let Ok(folder) = self.folder() {
            let _ = std::fs::remove_file(folder.join(STATE_FILENAME));
        }
        Ok(())
    }

    fn apply_original(original: &OriginalWallpaper) -> Result<()> {
        let api = NativeWallpaperApi::new()?;
        api.set_position(DESKTOP_WALLPAPER_POSITION(original.position))?;
        api.set_background_color(COLORREF(original.background_color))?;

        if let Some(slideshow) = &original.slideshow {
            match api.set_slideshow(slideshow) {
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
            return api.set_wallpaper(None, fallback);
        }

        for (monitor, _) in api.wallpapers() {
            let wallpaper = original
                .by_monitor
                .iter()
                .find(|(saved, _)| saved.eq_ignore_ascii_case(&monitor))
                .map(|(_, wallpaper)| wallpaper.as_str())
                .unwrap_or(fallback);
            if let Err(err) = api.set_wallpaper(Some(&monitor), wallpaper) {
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

/// Persisted while the native wallpaper is managed by Seelen UI.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SavedState {
    original: OriginalWallpaper,
    /// Last wallpaper set by us.
    applied: Option<String>,
}

impl SavedState {
    fn is_applied(&self, wallpaper: &str) -> bool {
        self.applied
            .as_deref()
            .is_some_and(|applied| paths_eq(applied, wallpaper))
    }
}

/// The user's wallpaper configuration before Seelen UI changed it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OriginalWallpaper {
    /// (monitor device path, wallpaper path), an empty path means no wallpaper (solid color).
    by_monitor: Vec<(String, String)>,
    position: i32,
    background_color: u32,
    /// `Some` if a slideshow was running.
    #[serde(default)]
    slideshow: Option<Slideshow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Slideshow {
    /// File system paths of the slideshow items (usually a single folder).
    items: Vec<String>,
    shuffle: bool,
    /// Milliseconds between images.
    interval: u32,
}

struct NativeWallpaperApi {
    inner: IDesktopWallpaper,
}

impl NativeWallpaperApi {
    fn new() -> Result<Self> {
        Ok(Self {
            inner: Com::create_instance(&DesktopWallpaper)?,
        })
    }

    /// (monitor device path, wallpaper path) of each monitor known by the wallpaper engine.
    fn wallpapers(&self) -> Vec<(String, String)> {
        let count = unsafe { self.inner.GetMonitorDevicePathCount() }.unwrap_or_default();
        (0..count)
            .filter_map(|index| {
                let monitor = take_pwstr(unsafe { self.inner.GetMonitorDevicePathAt(index) }.ok()?);
                let monitor_w = WindowsString::from_str(&monitor);
                // fails for monitors that are known but not attached
                let wallpaper =
                    take_pwstr(unsafe { self.inner.GetWallpaper(monitor_w.as_pcwstr()) }.ok()?);
                Some((monitor, wallpaper))
            })
            .collect()
    }

    fn snapshot(&self, by_monitor: Vec<(String, String)>) -> Result<OriginalWallpaper> {
        Ok(OriginalWallpaper {
            by_monitor,
            position: unsafe { self.inner.GetPosition()? }.0,
            background_color: unsafe { self.inner.GetBackgroundColor()? }.0,
            slideshow: self
                .slideshow()
                .inspect_err(|err| log::error!("Failed to read the slideshow: {err:?}"))
                .ok()
                .flatten(),
        })
    }

    /// `None` if no slideshow is running.
    fn slideshow(&self) -> Result<Option<Slideshow>> {
        let status = unsafe { self.inner.GetStatus()? };
        if status.0 & DSS_SLIDESHOW.0 == 0 {
            return Ok(None);
        }

        let array = unsafe { self.inner.GetSlideshow()? };
        let count = unsafe { array.GetCount()? };
        let mut items = Vec::new();
        for index in 0..count {
            let item = unsafe { array.GetItemAt(index)? };
            // items without a file system path (virtual folders) can't be persisted
            if let Ok(path) = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) } {
                items.push(take_pwstr(path));
            }
        }
        if items.is_empty() {
            return Ok(None);
        }

        let mut options = DESKTOP_SLIDESHOW_OPTIONS::default();
        let mut interval = 0u32;
        unsafe {
            self.inner
                .GetSlideshowOptions(&mut options, &mut interval)?
        };
        Ok(Some(Slideshow {
            items,
            shuffle: options.0 & DSO_SHUFFLEIMAGES.0 != 0,
            interval,
        }))
    }

    fn set_slideshow(&self, slideshow: &Slideshow) -> Result<()> {
        let pidls = slideshow
            .items
            .iter()
            .filter_map(|path| {
                let path = WindowsString::from_str(path);
                let mut pidl: *mut ITEMIDLIST = std::ptr::null_mut();
                unsafe { SHParseDisplayName(path.as_pcwstr(), None, &mut pidl, 0, None) }
                    .inspect_err(|err| log::warn!("Slideshow item not found: {err:?}"))
                    .ok()
                    .filter(|_| !pidl.is_null())
                    .map(|_| pidl)
            })
            .collect::<Vec<_>>();

        let result = (|| -> Result<()> {
            if pidls.is_empty() {
                return Err("None of the slideshow items exist anymore".into());
            }
            let const_pidls: Vec<*const ITEMIDLIST> = pidls.iter().map(|p| *p as _).collect();
            let array = unsafe { SHCreateShellItemArrayFromIDLists(&const_pidls)? };
            unsafe { self.inner.SetSlideshow(&array)? };
            let options = if slideshow.shuffle {
                DSO_SHUFFLEIMAGES
            } else {
                DESKTOP_SLIDESHOW_OPTIONS(0)
            };
            unsafe {
                self.inner
                    .SetSlideshowOptions(options, slideshow.interval)?
            };
            Ok(())
        })();

        for pidl in pidls {
            unsafe { ILFree(Some(pidl as _)) };
        }
        result
    }

    /// `monitor: None` sets it on all the monitors.
    fn set_wallpaper(&self, monitor: Option<&str>, wallpaper: &str) -> Result<()> {
        let monitor = monitor.map(WindowsString::from_str);
        let monitor = monitor
            .as_ref()
            .map_or(PCWSTR::null(), |monitor| monitor.as_pcwstr());
        let wallpaper = WindowsString::from_str(wallpaper);
        unsafe { self.inner.SetWallpaper(monitor, wallpaper.as_pcwstr())? };
        Ok(())
    }

    fn set_position(&self, position: DESKTOP_WALLPAPER_POSITION) -> Result<()> {
        unsafe { self.inner.SetPosition(position)? };
        Ok(())
    }

    fn set_background_color(&self, color: COLORREF) -> Result<()> {
        unsafe { self.inner.SetBackgroundColor(color)? };
        Ok(())
    }
}

/// Reads and frees a COM allocated string.
fn take_pwstr(value: PWSTR) -> String {
    if value.is_null() {
        return String::new();
    }
    let result = unsafe { value.to_string() }.unwrap_or_default();
    unsafe { CoTaskMemFree(Some(value.0 as _)) };
    result
}

fn paths_eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Files of Seelen UI (bundled or user resources, thumbnails, our black image) are never taken
/// as the user's own wallpaper, even if left by a previous session that didn't restore it.
fn is_seelen_owned(wallpaper: &str) -> bool {
    let lower = wallpaper.to_lowercase();
    if lower.contains("com.seelen.seelen-ui") || lower.contains("seelen.seelenui_") {
        return true;
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_string_lossy().to_lowercase()))
        .is_some_and(|dir| lower.starts_with(&dir))
}

/// 8x8 24 bits black bitmap (rows of 24 bytes, already 4-byte aligned).
fn black_bmp() -> Vec<u8> {
    const SIZE: u32 = 8;
    const DATA_SIZE: u32 = SIZE * SIZE * 3;
    const OFFSET: u32 = 14 + 40;

    let mut bmp = Vec::with_capacity((OFFSET + DATA_SIZE) as usize);
    // BITMAPFILEHEADER
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(OFFSET + DATA_SIZE).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&OFFSET.to_le_bytes());
    // BITMAPINFOHEADER
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(SIZE as i32).to_le_bytes());
    bmp.extend_from_slice(&(SIZE as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
    bmp.extend_from_slice(&24u16.to_le_bytes()); // bits per pixel
    bmp.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    bmp.extend_from_slice(&DATA_SIZE.to_le_bytes());
    bmp.extend_from_slice(&2835i32.to_le_bytes()); // 72 DPI
    bmp.extend_from_slice(&2835i32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    // pixels
    bmp.resize((OFFSET + DATA_SIZE) as usize, 0);
    bmp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_bmp_is_well_formed() {
        let bmp = black_bmp();
        assert_eq!(&bmp[0..2], b"BM");
        assert_eq!(
            u32::from_le_bytes(bmp[2..6].try_into().unwrap()) as usize,
            bmp.len()
        );
        assert_eq!(bmp.len(), 54 + 8 * 8 * 3);
    }

    #[test]
    fn seelen_owned_paths() {
        assert!(is_seelen_owned(
            r"C:\Users\x\AppData\Roaming\com.seelen.seelen-ui\wallpapers\a\thumbnail.jpg"
        ));
        assert!(!is_seelen_owned(
            r"C:\Windows\Web\Wallpaper\Windows\img0.jpg"
        ));
    }
}
