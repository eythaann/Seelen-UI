#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::path::PathBuf;

use crate::error::Result;

/// Per-user local (non roaming) data folder.\
/// Windows: `%LOCALAPPDATA%`
pub fn local_data_dir() -> Result<PathBuf> {
    imp::local_data_dir()
}

/// Root folder of the OS installation.\
/// Windows: `C:\Windows`
pub fn os_root_dir() -> Result<PathBuf> {
    imp::os_root_dir()
}

/// Folder containing the OS binaries.\
/// Windows: `C:\Windows\System32`
pub fn os_bin_dir() -> Result<PathBuf> {
    imp::os_bin_dir()
}

/// Per-user folder of application launchers/shortcuts, preferred over [`system_apps_dir`].\
/// Windows: `%APPDATA%\Microsoft\Windows\Start Menu`
pub fn user_apps_dir() -> Result<PathBuf> {
    imp::user_apps_dir()
}

/// System-wide folder of application launchers/shortcuts, only a fallback of [`user_apps_dir`].\
/// Windows: `%PROGRAMDATA%\Microsoft\Windows\Start Menu`
pub fn system_apps_dir() -> Result<PathBuf> {
    imp::system_apps_dir()
}
