use std::path::PathBuf;

use windows::Win32::UI::Shell::{
    FOLDERID_CommonStartMenu, FOLDERID_LocalAppData, FOLDERID_StartMenu, FOLDERID_System,
    FOLDERID_Windows,
};

use crate::{error::Result, windows_api::WindowsApi};

use super::PathsApi;

pub struct WindowsPaths;

impl PathsApi for WindowsPaths {
    fn local_data_dir() -> Result<PathBuf> {
        WindowsApi::known_folder(FOLDERID_LocalAppData)
    }

    fn os_root_dir() -> Result<PathBuf> {
        WindowsApi::known_folder(FOLDERID_Windows)
    }

    fn os_bin_dir() -> Result<PathBuf> {
        WindowsApi::known_folder(FOLDERID_System)
    }

    fn user_apps_dir() -> Result<PathBuf> {
        WindowsApi::known_folder(FOLDERID_StartMenu)
    }

    fn system_apps_dir() -> Result<PathBuf> {
        WindowsApi::known_folder(FOLDERID_CommonStartMenu)
    }
}
