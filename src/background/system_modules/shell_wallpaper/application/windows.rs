use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use notify_debouncer_full::{
    DebounceEventResult, Debouncer, FileIdMap, new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
};
use windows::Win32::UI::WindowsAndMessaging::WM_SETTINGCHANGE;

use crate::{
    error::Result,
    platform::shell,
    system_modules::shell_wallpaper::domain::{ShellWallpaperSysEvent, ShellWallpaperSystem},
    utils::lock_free::TracedMutex,
    windows_api::event_window::subscribe_to_background_window,
};

const FOLDER_WATCHER_DEBOUNCE: Duration = Duration::from_millis(300);

type EventCallback = Arc<dyn Fn(ShellWallpaperSysEvent) + Send + Sync>;

pub struct WindowsShellWallpaper {
    /// `None` until `init` is called.
    on_event: Option<EventCallback>,
    watcher: TracedMutex<Option<FolderWatcher>>,
}

impl ShellWallpaperSystem for WindowsShellWallpaper {
    fn create() -> Self {
        Self {
            on_event: None,
            watcher: TracedMutex::new(None),
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(ShellWallpaperSysEvent) + Send + Sync + 'static,
    {
        let on_event: EventCallback = Arc::new(on_event);
        self.on_event = Some(on_event.clone());

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

        let previous = {
            let mut watcher = self.watcher.lock();
            if watcher.as_ref().map(|w| w.folder.as_path()) == folder {
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
