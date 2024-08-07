use itertools::Itertools;
use lazy_static::lazy_static;
use notify::{RecursiveMode, Watcher};
use parking_lot::Mutex;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tauri::{Emitter, Manager};

use crate::{error_handler::Result, log_error, seelen::get_app_handle};

use super::domain::Theme;

lazy_static! {
    pub static ref THEME_MANAGER: Arc<Mutex<ThemeManager>> = Arc::new(Mutex::new(
        ThemeManager::new().expect("Failed to create theme manager")
    ));
}

pub struct ThemeManager {
    themes: HashMap<String, Theme>,
}

impl ThemeManager {
    fn new() -> Result<Self> {
        let mut manager = Self {
            themes: HashMap::new(),
        };
        manager.load_themes()?;
        manager.start_listener()?;
        Ok(manager)
    }

    pub fn themes(&self) -> &HashMap<String, Theme> {
        &self.themes
    }

    fn start_listener(&mut self) -> Result<()> {
        let (tx, rx) = crossbeam_channel::unbounded();

        let mut watcher = notify::recommended_watcher(tx)?;

        let handle = get_app_handle();
        let user_path = handle.path().app_data_dir()?.join("themes");
        let resources_path = handle.path().resource_dir()?.join("static/themes");
        watcher.watch(&user_path, RecursiveMode::Recursive)?;
        watcher.watch(&resources_path, RecursiveMode::Recursive)?;

        std::thread::spawn(move || {
            let _watcher = watcher;
            for event in rx {
                match event {
                    Ok(event) => {
                        log::info!("Theme changed: {:?} {:?}", event.paths, event.kind);
                        let mut manager = THEME_MANAGER.lock();
                        log_error!(manager.load_themes());
                        log_error!(manager.emit_themes());
                    }
                    Err(e) => log::error!("Themes watcher error: {:?}", e),
                }
            }
        });

        log::info!("Themes watcher started");
        Ok(())
    }

    fn load_theme_from_file(path: PathBuf) -> Result<Theme> {
        match path.extension() {
            Some(ext) if ext == "yml" || ext == "yaml" => {
                Ok(serde_yaml::from_str(&std::fs::read_to_string(&path)?)?)
            }
            _ => Err("Invalid theme file extension".into()),
        }
    }

    fn load_theme_from_dir(path: PathBuf) -> Result<Theme> {
        let file = path.join("theme.yml");
        if !file.exists() {
            return Err("theme.yml not found".into());
        }

        let mut theme = Self::load_theme_from_file(file)?;

        if path.join("theme.weg.css").exists() {
            theme.styles.weg = std::fs::read_to_string(path.join("theme.weg.css"))?;
        }

        if path.join("theme.toolbar.css").exists() {
            theme.styles.toolbar = std::fs::read_to_string(path.join("theme.toolbar.css"))?;
        }

        if path.join("theme.wm.css").exists() {
            theme.styles.wm = std::fs::read_to_string(path.join("theme.wm.css"))?;
        }

        Ok(theme)
    }

    fn emit_themes(&self) -> Result<()> {
        let handle = get_app_handle();
        handle.emit("themes", &self.themes().values().collect_vec())?;
        Ok(())
    }

    fn load_themes(&mut self) -> Result<()> {
        let handle = get_app_handle();
        let user_path = handle.path().app_data_dir()?.join("themes");
        let resources_path = handle.path().resource_dir()?.join("static/themes");
        let entries = std::fs::read_dir(&resources_path)?.chain(std::fs::read_dir(&user_path)?);
        for entry in entries.flatten() {
            let path = entry.path();
            let theme = if path.is_dir() {
                Self::load_theme_from_dir(path)
            } else {
                Self::load_theme_from_file(path)
            };
            match theme {
                Ok(mut theme) => {
                    theme.info.filename = entry.file_name().to_string_lossy().to_string();
                    self.themes.insert(theme.info.filename.clone(), theme);
                }
                Err(err) => log::error!("Failed to load theme ({:?}): {:?}", entry.path(), err),
            }
        }
        Ok(())
    }
}
