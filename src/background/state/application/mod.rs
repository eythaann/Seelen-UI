use lazy_static::lazy_static;
use notify::{RecursiveMode, Watcher};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::{error_handler::Result, log_error, seelen::get_app_handle};

use super::domain::WegItems;

lazy_static! {
    pub static ref FULL_STATE: Arc<Mutex<FullState>> = Arc::new(Mutex::new(
        FullState::new().expect("Failed to create placeholders manager")
    ));
}

pub struct FullState {
    handle: AppHandle<tauri::Wry>,
    weg_items: WegItems,
}

impl FullState {
    fn new() -> Result<Self> {
        let mut manager = Self {
            handle: get_app_handle(),
            weg_items: serde_yaml::Value::Null,
        };
        manager.load_all()?;
        manager.start_listeners()?;
        Ok(manager)
    }

    pub fn weg_items(&self) -> &WegItems {
        &self.weg_items
    }

    fn start_listeners(&mut self) -> Result<()> {
        let (tx, rx) = crossbeam_channel::unbounded();

        let mut watcher = notify::recommended_watcher(tx)?;

        let data_dir = self.handle.path().app_data_dir()?;
        let weg_items_path = data_dir.join("seelenweg_items.yaml");

        watcher.watch(&data_dir, RecursiveMode::Recursive)?;

        std::thread::spawn(move || {
            let _watcher = watcher;
            for event in rx {
                match event {
                    Ok(event) => {
                        if event.paths.contains(&weg_items_path) {
                            log::info!("Weg Items changed: {:?}", weg_items_path);
                            let mut manager = FULL_STATE.lock();
                            log_error!(manager.load_weg_items());
                            log_error!(manager.emit_weg_items());
                        }
                    }
                    Err(e) => log::error!("Placeholder watcher error: {:?}", e),
                }
            }
        });

        log::info!("Seelen UI Data Watcher started!");
        Ok(())
    }

    fn load_weg_items(&mut self) -> Result<()> {
        let dir = self.handle.path().app_data_dir()?;
        let path = dir.join("seelenweg_items.yaml");

        self.weg_items = if !path.exists() {
            serde_yaml::Value::Null
        } else {
            serde_yaml::from_str(&std::fs::read_to_string(&path)?)?
        };

        Ok(())
    }

    fn load_all(&mut self) -> Result<()> {
        self.load_weg_items()?;
        Ok(())
    }

    fn emit_weg_items(&self) -> Result<()> {
        let handle = get_app_handle();
        handle.emit("weg-items", self.weg_items())?;
        Ok(())
    }
}
