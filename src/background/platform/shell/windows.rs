use std::path::{Path, PathBuf};

use crate::{
    error::Result,
    utils::constants::SEELEN_COMMON,
    windows_api::{WindowsApi, input::Keyboard, msix::MsixAppsManager},
};

/// `ShellExecute` inside explorer.exe, falling back to our own process.
pub fn execute(
    program: &str,
    args: Option<&str>,
    working_dir: Option<&Path>,
    elevated: bool,
) -> Result<()> {
    WindowsApi::execute(
        program.to_owned(),
        args.map(str::to_owned),
        working_dir.map(Path::to_path_buf),
        elevated,
    )
}

pub fn reveal(path: &Path) -> Result<()> {
    std::process::Command::new(SEELEN_COMMON.system_dir().join("explorer.exe"))
        .arg("/select,")
        .arg(path)
        .spawn()?;
    Ok(())
}

/// Only packaged (MSIX/AppX) apps are resolved, other AUMIDs fail.
pub fn app_executable(app_id: &str) -> Result<Option<PathBuf>> {
    MsixAppsManager::instance().get_app_path(app_id)
}

pub fn file_app_id(path: &Path) -> Result<String> {
    WindowsApi::get_file_umid(path)
}

pub fn file_display_name(path: &Path) -> Result<String> {
    WindowsApi::get_executable_display_name(path)
}

/// There is no API to open it, pressing the Windows key is the only reliable trick.
pub fn open_native_start_menu() -> Result<()> {
    Keyboard::new().send_keys("{win}")
}
