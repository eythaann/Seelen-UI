#[cfg(windows)]
mod windows;

use std::path::PathBuf;

use crate::error::Result;

#[cfg(windows)]
type Native = windows::WindowsPaths;

/// Contract to be implemented by each OS.
trait PathsApi {
    fn local_data_dir() -> Result<PathBuf>;
    fn os_root_dir() -> Result<PathBuf>;
    fn os_bin_dir() -> Result<PathBuf>;
    fn user_apps_dir() -> Result<PathBuf>;
    fn system_apps_dir() -> Result<PathBuf>;
}

/// Per-user local (non roaming) data folder.\
/// Windows: `%LOCALAPPDATA%`
pub fn local_data_dir() -> Result<PathBuf> {
    Native::local_data_dir()
}

/// Root folder of the OS installation.\
/// Windows: `C:\Windows`
pub fn os_root_dir() -> Result<PathBuf> {
    Native::os_root_dir()
}

/// Folder containing the OS binaries.\
/// Windows: `C:\Windows\System32`
pub fn os_bin_dir() -> Result<PathBuf> {
    Native::os_bin_dir()
}

/// Per-user folder of application launchers/shortcuts, preferred over [`system_apps_dir`].\
/// Windows: `%APPDATA%\Microsoft\Windows\Start Menu`
pub fn user_apps_dir() -> Result<PathBuf> {
    Native::user_apps_dir()
}

/// System-wide folder of application launchers/shortcuts, only a fallback of [`user_apps_dir`].\
/// Windows: `%PROGRAMDATA%\Microsoft\Windows\Start Menu`
pub fn system_apps_dir() -> Result<PathBuf> {
    Native::system_apps_dir()
}
