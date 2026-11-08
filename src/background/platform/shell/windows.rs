use std::path::{Path, PathBuf};

use widestring::U16CStr;
use windows::Win32::{
    Foundation::MAX_PATH,
    UI::WindowsAndMessaging::{
        SPI_GETDESKWALLPAPER, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
    },
};

use crate::{
    error::Result,
    utils::constants::SEELEN_COMMON,
    windows_api::{
        WindowsApi, input::Keyboard, msix::MsixAppsManager, string_utils::WindowsString,
    },
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

pub fn wallpaper() -> Result<PathBuf> {
    let mut path = [0_u16; MAX_PATH as usize];
    unsafe {
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            MAX_PATH,
            Some(path.as_mut_ptr() as _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )?;
    }
    Ok(PathBuf::from(
        U16CStr::from_slice_truncate(&path)?
            .to_ustring()
            .to_string_lossy(),
    ))
}

pub fn set_wallpaper(path: &Path) -> Result<()> {
    let mut path = WindowsString::from(path.to_path_buf());
    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            MAX_PATH,
            Some(path.as_mut_slice().as_mut_ptr() as _),
            SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
        )?;
    }
    Ok(())
}
