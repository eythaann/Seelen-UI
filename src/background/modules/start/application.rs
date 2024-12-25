use std::path::{Path, PathBuf};

use arc_swap::ArcSwap;
use lazy_static::lazy_static;
use tauri::{path::BaseDirectory, Manager};

use crate::{error_handler::Result, seelen::get_app_handle, windows_api::WindowsApi};

use super::domain::StartMenuItem;

lazy_static! {
    pub static ref START_MENU_ITEMS: ArcSwap<Vec<StartMenuItem>> =
        ArcSwap::from_pointee(StartMenuManager::get_items().unwrap());
}

pub struct StartMenuManager {}

impl StartMenuManager {
    pub fn common_items_path() -> PathBuf {
        PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs")
    }

    pub fn user_items_path() -> PathBuf {
        get_app_handle()
            .path()
            .resolve(
                r"Microsoft\Windows\Start Menu\Programs",
                BaseDirectory::Data,
            )
            .expect("Failed to resolve user start menu path")
    }

    fn _get_items(dir: &Path) -> Result<Vec<StartMenuItem>> {
        let mut items = Vec::new();
        for entry in std::fs::read_dir(dir)?.flatten() {
            let file_type = entry.file_type()?;
            let path = entry.path();
            if file_type.is_dir() {
                items.extend(Self::_get_items(&path)?);
                continue;
            }
            if file_type.is_file() {
                items.push(StartMenuItem {
                    umid: WindowsApi::get_file_umid(&path).ok(),
                    path,
                })
            }
        }
        Ok(items)
    }

    pub fn get_items() -> Result<Vec<StartMenuItem>> {
        let mut items = vec![];
        items.extend(Self::_get_items(&Self::common_items_path())?);
        items.extend(Self::_get_items(&Self::user_items_path())?);
        Ok(items)
    }
}
