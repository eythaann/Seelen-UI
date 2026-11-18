use std::path::{Path, PathBuf};

use seelen_core::state::NativeWallpaperSnapshot;

use crate::error::Result;

/// OS contract of the native shell wallpaper module.
pub trait ShellWallpaperSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening the OS notifications that may mean the wallpaper changed, reported
    /// through `on_event`. Called once; the subscription lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(ShellWallpaperSysEvent) + Send + Sync + 'static;

    /// Image currently set as the desktop wallpaper, empty when there is none (solid color).
    fn wallpaper(&self) -> Result<PathBuf>;

    /// Watches the given folder (non recursive) replacing the previous watch, `None` stops it.
    /// Needed because some apps overwrite the same wallpaper file in place.
    fn watch_folder(&self, folder: Option<&Path>) -> Result<()>;
}

/// Raw notifications of the OS layer, the wallpaper may have changed and should be read again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellWallpaperSysEvent {
    SettingChanged,
    FolderChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellWallpaperEvent {
    Changed { snapshot: NativeWallpaperSnapshot },
}
