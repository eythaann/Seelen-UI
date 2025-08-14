use std::path::{Path, PathBuf};
use std::sync::Arc;

use arc_swap::ArcSwap;
use lazy_static::lazy_static;
use seelen_core::system_state::StartMenuItem;
use tauri::{path::BaseDirectory, Manager};

use crate::{
    app::get_app_handle, error::Result, log_error, utils::constants::SEELEN_COMMON,
    windows_api::WindowsApi,
};

lazy_static! {
    pub static ref START_MENU_MANAGER: ArcSwap<StartMenuManager> = ArcSwap::from_pointee({
        let mut manager = StartMenuManager::new();
        manager.init().unwrap();
        manager
    });
}

pub struct StartMenuManager {
    pub list: Vec<StartMenuItem>,
    cache_path: PathBuf,
}

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

    pub fn new() -> StartMenuManager {
        StartMenuManager {
            list: Vec::new(),
            cache_path: SEELEN_COMMON.app_cache_dir().join("start_menu_v2.json"),
        }
    }

    fn init(&mut self) -> Result<()> {
        if self.cache_path.exists() {
            match self.load_cache() {
                Ok(_) => {
                    // refresh without blocking
                    std::thread::spawn(|| {
                        let mut menu = StartMenuManager::new();
                        log_error!(menu.read_start_menu_folders());
                        log_error!(menu.store_cache());
                        START_MENU_MANAGER.swap(Arc::new(menu));
                    });
                    return Ok(());
                }
                Err(e) => {
                    log::error!("Failed to load start menu cache: {e}");
                }
            }
        }

        self.read_start_menu_folders()?;
        self.store_cache()?;
        Ok(())
    }

    pub fn get_by_target(&self, target: &Path) -> Option<&StartMenuItem> {
        self.list
            .iter()
            .find(|item| item.target.as_ref().is_some_and(|t| t == target))
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchiconresource
    pub fn get_by_file_umid(&self, umid: &str) -> Option<&StartMenuItem> {
        self.list.iter().find(|item| {
            if let Some(item_umid) = &item.umid {
                return item_umid == umid;
            }
            if let Some(target) = &item.target {
                // some apps registered as media player as example use the process name as umid
                return target.ends_with(umid);
            }
            false
        })
    }

    pub fn store_cache(&self) -> Result<()> {
        let file = std::fs::File::create(&self.cache_path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.list)?;
        Ok(())
    }

    pub fn load_cache(&mut self) -> Result<()> {
        let file = std::fs::File::open(&self.cache_path)?;
        let reader = std::io::BufReader::new(file);
        self.list = serde_json::from_reader(reader)?;
        Ok(())
    }

    fn _get_items(dir: &Path) -> Result<Vec<StartMenuItem>> {
        let mut items = Vec::new();
        for entry in std::fs::read_dir(dir)?.flatten() {
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                items.extend(Self::_get_items(&path)?);
                continue;
            }
            if file_type.is_file() {
                let target = WindowsApi::resolve_lnk_target(&path).ok().map(|(t, _)| t);
                items.push(StartMenuItem {
                    umid: WindowsApi::get_file_umid(&path).ok(),
                    path,
                    target,
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
