use itertools::Itertools;
use lazy_static::lazy_static;
use notify::{RecursiveMode, Watcher};
use parking_lot::Mutex;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tauri::{Emitter, Manager};

use crate::{error_handler::Result, log_error, seelen::get_app_handle};

use super::domain::Placeholder;

lazy_static! {
    pub static ref PLACEHOLDERS_MANAGER: Arc<Mutex<ToolbarLayoutManager>> = Arc::new(Mutex::new(
        ToolbarLayoutManager::new().expect("Failed to create placeholders manager")
    ));
}

pub struct ToolbarLayoutManager {
    placeholders: HashMap<String, Placeholder>,
}

impl ToolbarLayoutManager {
    fn new() -> Result<Self> {
        let mut manager = Self {
            placeholders: HashMap::new(),
        };
        manager.load_placeholders()?;
        manager.start_listener()?;
        Ok(manager)
    }

    pub fn placeholders(&self) -> &HashMap<String, Placeholder> {
        &self.placeholders
    }

    fn start_listener(&mut self) -> Result<()> {
        let (tx, rx) = crossbeam_channel::unbounded();

        let mut watcher = notify::recommended_watcher(tx)?;

        let handle = get_app_handle();
        let user_path = handle.path().app_data_dir()?.join("placeholders");
        let resources_path = handle.path().resource_dir()?.join("static/placeholders");
        watcher.watch(&user_path, RecursiveMode::Recursive)?;
        watcher.watch(&resources_path, RecursiveMode::Recursive)?;

        std::thread::spawn(move || {
            let _watcher = watcher;
            for event in rx {
                match event {
                    Ok(event) => {
                        log::info!("Placeholder changed: {:?} {:?}", event.paths, event.kind);
                        let mut manager = PLACEHOLDERS_MANAGER.lock();
                        log_error!(manager.load_placeholders());
                        log_error!(manager.emit_placeholders());
                    }
                    Err(e) => log::error!("Placeholder watcher error: {:?}", e),
                }
            }
        });

        log::info!("Placeholders watcher started");
        Ok(())
    }

    fn load_placeholder_from_file(path: PathBuf) -> Result<Placeholder> {
        match path.extension() {
            Some(ext) if ext == "yml" || ext == "yaml" => {
                Ok(serde_yaml::from_str(&std::fs::read_to_string(&path)?)?)
            }
            _ => Err("Invalid placeholder file extension".into()),
        }
    }

    fn emit_placeholders(&self) -> Result<()> {
        let handle = get_app_handle();
        handle.emit("placeholders", &self.placeholders().values().collect_vec())?;
        Ok(())
    }

    fn load_placeholders(&mut self) -> Result<()> {
        let handle = get_app_handle();
        let user_path = handle.path().app_data_dir()?.join("placeholders");
        let resources_path = handle.path().resource_dir()?.join("static/placeholders");
        let entries = std::fs::read_dir(&resources_path)?.chain(std::fs::read_dir(&user_path)?);
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let placeholder = Self::load_placeholder_from_file(path);

            match placeholder {
                Ok(mut placeholder) => {
                    placeholder.info.filename = entry.file_name().to_string_lossy().to_string();
                    self.placeholders
                        .insert(placeholder.info.filename.clone(), placeholder);
                }
                Err(err) => {
                    log::error!("Failed to load placeholder ({:?}): {:?}", entry.path(), err)
                }
            }
        }
        Ok(())
    }
}
