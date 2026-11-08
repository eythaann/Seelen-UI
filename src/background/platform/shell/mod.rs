//! OS shell integration: launching programs, documents and URLs, installed apps and the native
//! desktop wallpaper.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::path::{Path, PathBuf};

use crate::error::Result;

/// Launches a program, document or URL with the OS shell, detached from our process (it doesn't
/// inherit our console, environment nor elevation).
pub fn execute(
    program: &str,
    args: Option<&str>,
    working_dir: Option<&Path>,
    elevated: bool,
) -> Result<()> {
    imp::execute(program, args, working_dir, elevated)
}

/// Opens a file or URL with its default handler.
pub fn open(target: &str) -> Result<()> {
    execute(target, None, None, false)
}

/// Shows the file selected in the OS file manager.
pub fn reveal(path: &Path) -> Result<()> {
    imp::reveal(path)
}

/// Executable of an installed app identified by its OS app id (the `umid` of the weg items).
/// `None` when the app is known but has no executable of its own.
pub fn app_executable(app_id: &str) -> Result<Option<PathBuf>> {
    imp::app_executable(app_id)
}

/// OS app id assigned to a file (Windows: the AUMID of a shortcut's property store).
pub fn file_app_id(path: &Path) -> Result<String> {
    imp::file_app_id(path)
}

/// Human readable name of a program or shortcut, as shown by the OS file manager.
pub fn file_display_name(path: &Path) -> Result<String> {
    imp::file_display_name(path)
}

/// Opens the start menu of the OS shell.
pub fn open_native_start_menu() -> Result<()> {
    imp::open_native_start_menu()
}

/// Image currently set as the OS desktop wallpaper.
pub fn wallpaper() -> Result<PathBuf> {
    imp::wallpaper()
}

/// Sets the OS desktop wallpaper.
pub fn set_wallpaper(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err("File not found".into());
    }
    imp::set_wallpaper(path)
}
