use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use notify_debouncer_full::{
    DebounceEventResult, Debouncer, FileIdMap, new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
};
use windows::Win32::UI::{Shell::FOLDERID_RoamingAppData, WindowsAndMessaging::WM_SETTINGCHANGE};

use crate::{
    error::{Result, ResultLogExt},
    platform::shell,
    system_modules::shell_wallpaper::domain::{ShellWallpaperSysEvent, ShellWallpaperSystem},
    utils::lock_free::TracedMutex,
    windows_api::{WindowsApi, event_window::subscribe_to_background_window},
};

const FOLDER_WATCHER_DEBOUNCE: Duration = Duration::from_millis(300);

type EventCallback = Arc<dyn Fn(ShellWallpaperSysEvent) + Send + Sync>;

pub struct WindowsShellWallpaper {
    /// `None` until `init` is called.
    on_event: Option<EventCallback>,
    /// Always watched: Windows writes here every picture it applies (`TranscodedWallpaper`), even
    /// when the previous wallpaper lived in another folder (e.g. leaving Windows Spotlight).
    themes_watcher: Option<FolderWatcher>,
    /// Folder of the current wallpaper, when it is not the themes folder.
    watcher: TracedMutex<Option<FolderWatcher>>,
}

impl ShellWallpaperSystem for WindowsShellWallpaper {
    fn create() -> Self {
        Self {
            on_event: None,
            themes_watcher: None,
            watcher: TracedMutex::new(None),
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(ShellWallpaperSysEvent) + Send + Sync + 'static,
    {
        let on_event: EventCallback = Arc::new(on_event);
        self.on_event = Some(on_event.clone());

        // A missing themes folder only degrades detection, setting changes are still reported.
        self.themes_watcher = themes_folder()
            .and_then(|folder| FolderWatcher::create(folder, on_event.clone()))
            .ok_logged();

        // Every setting change is forwarded instead of trusting `SPI_SETDESKWALLPAPER` to be the
        // reported parameter: reading the wallpaper again is cheap and only emits on a real change.
        subscribe_to_background_window(move |msg, _w_param, _l_param| {
            if msg == WM_SETTINGCHANGE {
                on_event(ShellWallpaperSysEvent::SettingChanged);
            }
            Ok(())
        });
        Ok(())
    }

    fn wallpaper(&self) -> Result<PathBuf> {
        shell::wallpaper()
    }

    fn watch_folder(&self, folder: Option<&Path>) -> Result<()> {
        let on_event = self
            .on_event
            .clone()
            .ok_or("Shell wallpaper module is not initialized")?;

        // Already covered by the permanent watcher.
        let folder = folder.filter(|folder| {
            !self
                .themes_watcher
                .as_ref()
                .is_some_and(|themes| same_folder(&themes.folder, folder))
        });

        let previous = {
            let mut watcher = self.watcher.lock();
            let current = watcher.as_ref().map(|w| w.folder.as_path());
            let unchanged = match (current, folder) {
                (Some(current), Some(folder)) => same_folder(current, folder),
                (None, None) => true,
                _ => false,
            };
            if unchanged {
                return Ok(());
            }
            watcher.take()
        };
        // Dropped outside the lock, the debouncer drop only signals its thread to stop.
        drop(previous);

        let Some(folder) = folder else {
            return Ok(());
        };
        let created = FolderWatcher::create(folder.to_path_buf(), on_event)?;
        *self.watcher.lock() = Some(created);
        Ok(())
    }
}

struct FolderWatcher {
    folder: PathBuf,
    _debouncer: Debouncer<ReadDirectoryChangesWatcher, FileIdMap>,
}

impl FolderWatcher {
    fn create(folder: PathBuf, on_event: EventCallback) -> Result<Self> {
        let mut debouncer = new_debouncer(
            FOLDER_WATCHER_DEBOUNCE,
            None,
            move |result: DebounceEventResult| match result {
                Ok(_) => on_event(ShellWallpaperSysEvent::FolderChanged),
                Err(errors) => log::warn!("Wallpaper folder watcher errors: {errors:?}"),
            },
        )?;
        debouncer.watch(&folder, RecursiveMode::NonRecursive)?;
        Ok(Self {
            folder,
            _debouncer: debouncer,
        })
    }
}

/// `%APPDATA%\Microsoft\Windows\Themes`
fn themes_folder() -> Result<PathBuf> {
    Ok(WindowsApi::known_folder(FOLDERID_RoamingAppData)?.join(r"Microsoft\Windows\Themes"))
}

/// Windows paths are case insensitive, and the reported wallpaper path changes its casing
/// depending on the app that set it.
fn same_folder(a: &Path, b: &Path) -> bool {
    a.as_os_str().eq_ignore_ascii_case(b.as_os_str())
}
