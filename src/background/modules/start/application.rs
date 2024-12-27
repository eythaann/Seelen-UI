use std::path::{Path, PathBuf};

use arc_swap::ArcSwap;
use lazy_static::lazy_static;
use tauri::{path::BaseDirectory, Manager};

use crate::{
    error_handler::Result, seelen::get_app_handle, utils::constants::SEELEN_COMMON,
    windows_api::WindowsApi,
};

use super::domain::StartMenuItem;

lazy_static! {
    pub static ref START_MENU_ITEMS: ArcSwap<Vec<StartMenuItem>> = ArcSwap::from_pointee({
        let mut manager = StartMenuManager::new();
        manager.init().unwrap();
        manager.list
    });
}

pub struct StartMenuManager {
    list: Vec<StartMenuItem>,
    cache_path: PathBuf,
}

impl StartMenuManager {
    pub fn new() -> StartMenuManager {
        StartMenuManager {
            list: Vec::new(),
            cache_path: SEELEN_COMMON.app_cache_dir().join("start_menu.json"),
        }
    }

    fn init(&mut self) -> Result<()> {
        if self.cache_path.exists() {
            self.load_cache()?;
        } else {
            self.read_start_menu_folders()?;
            self.store_cache()?;
        }
        Ok(())
    }

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

    pub fn store_cache(&self) -> Result<()> {
        let writer = std::fs::File::create(&self.cache_path)?;
        serde_json::to_writer(writer, &self.list)?;
        Ok(())
    }

    pub fn load_cache(&mut self) -> Result<()> {
        let reader = std::fs::File::open(&self.cache_path)?;
        self.list = serde_json::from_reader(reader)?;
        Ok(())
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

    pub fn read_start_menu_folders(&mut self) -> Result<()> {
        let mut items = vec![];
        items.extend(Self::_get_items(&Self::common_items_path())?);
        items.extend(Self::_get_items(&Self::user_items_path())?);
        self.list = items;
        Ok(())
    }
}
