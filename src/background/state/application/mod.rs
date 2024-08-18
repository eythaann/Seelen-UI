mod apps_config;

use arc_swap::ArcSwap;
use getset::Getters;
use itertools::Itertools;
use lazy_static::lazy_static;
use notify::{RecursiveMode, Watcher};
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    error_handler::Result,
    log_error,
    modules::cli::domain::Resource,
    seelen::{get_app_handle, SEELEN},
    trace_lock,
    utils::{is_virtual_desktop_supported, spawn_named_thread},
    windows_api::WindowsApi,
};

use super::{
    application::apps_config::REGEX_IDENTIFIERS,
    domain::{AppConfig, Placeholder, Settings, Theme, WegItems},
};

lazy_static! {
    pub static ref FULL_STATE: Arc<ArcSwap<FullState>> = Arc::new(ArcSwap::from_pointee(
        FullState::new().expect("Failed to create State Manager")
    ));
}

#[derive(Getters, Debug, Clone, Serialize)]
pub struct FullState {
    #[serde(skip)]
    handle: AppHandle<tauri::Wry>,
    #[serde(skip)]
    data_dir: PathBuf,
    #[serde(skip)]
    resources_dir: PathBuf,
    // ======== data ========
    #[getset(get = "pub")]
    settings: Settings,
    #[getset(get = "pub")]
    settings_by_app: VecDeque<AppConfig>,
    #[getset(get = "pub")]
    themes: HashMap<String, Theme>,
    #[getset(get = "pub")]
    placeholders: HashMap<String, Placeholder>,
    #[getset(get = "pub")]
    weg_items: WegItems,
}

static FILE_LISTENER_PAUSED: AtomicBool = AtomicBool::new(false);

impl FullState {
    fn new() -> Result<Self> {
        let handle = get_app_handle();
        let mut manager = Self {
            data_dir: handle.path().app_data_dir()?,
            resources_dir: handle.path().resource_dir()?,
            handle,
            // ======== data ========
            settings: Settings::default(),
            settings_by_app: VecDeque::new(),
            themes: HashMap::new(),
            placeholders: HashMap::new(),
            weg_items: serde_yaml::Value::Null,
        };
        manager.load_all()?;
        manager.start_listeners()?;
        Ok(manager)
    }

    /// shorthand of `FullState::clone` on Arc reference
    pub fn cloned(&self) -> Self {
        self.clone()
    }

    /// store `self` as the static `FULL_STATE` instance
    pub fn store(self) {
        FULL_STATE.store(Arc::new(self));
    }

    pub fn settings_path(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }

    fn process_event(&mut self, event: notify::Event) -> Result<()> {
        let weg_items_path = self.data_dir.join("seelenweg_items.yaml");

        let user_themes = self.data_dir.join("themes");
        let bundled_themes = self.resources_dir.join("static/themes");

        let user_placeholders = self.data_dir.join("placeholders");
        let bundled_placeholders = self.resources_dir.join("static/placeholders");

        let user_app_configs = self.data_dir.join("applications.yml");
        let bundled_app_configs = self.resources_dir.join("static/apps_templates");

        if event.paths.contains(&weg_items_path) {
            log::info!("Weg Items changed: {:?}", event.paths);
            self.load_weg_items()?;
            self.emit_weg_items()?;
        }

        if event.paths.contains(&self.settings_path()) {
            log::info!("Seelen Settings changed: {:?}", event.paths);
            self.load_settings()?;
            self.emit_settings()?;
        }

        if event
            .paths
            .iter()
            .any(|p| p.starts_with(&user_themes) || p.starts_with(&bundled_themes))
        {
            log::info!("Theme changed: {:?}", event.paths);
            self.load_themes()?;
            self.emit_themes()?;
        }

        if event
            .paths
            .iter()
            .any(|p| p.starts_with(&user_placeholders) || p.starts_with(&bundled_placeholders))
        {
            log::info!("Placeholder changed: {:?}", event.paths);
            self.load_placeholders()?;
            self.emit_placeholders()?;
        }

        if event
            .paths
            .iter()
            .any(|p| p.starts_with(&user_app_configs) || p.starts_with(&bundled_app_configs))
        {
            log::info!("Specific App Configuration changed: {:?}", event.paths);
            self.load_settings_by_app()?;
            self.emit_settings_by_app()?;
        }

        Ok(())
    }

    fn start_listeners(&mut self) -> Result<()> {
        let (tx, rx) = crossbeam_channel::unbounded();

        let mut watcher = notify::recommended_watcher(tx)?;

        watcher.watch(&self.data_dir, RecursiveMode::Recursive)?;
        watcher.watch(&self.resources_dir, RecursiveMode::Recursive)?;

        // Todo I think this thread is unnecessary, just try saving the watcher obj in the instance.
        spawn_named_thread("Files Watcher", move || {
            let _watcher = watcher;
            for event in rx {
                match event {
                    Ok(event) => {
                        if !FILE_LISTENER_PAUSED.load(Ordering::Acquire) {
                            let mut state = FULL_STATE.load().cloned();
                            log_error!(state.process_event(event));
                            state.store();
                        }
                    }
                    Err(e) => log::error!("Seelen UI Data Watcher error: {:?}", e),
                }
            }
        })?;

        log::trace!("Seelen UI Data Watcher started!");
        Ok(())
    }

    fn load_settings(&mut self) -> Result<()> {
        let path = self.settings_path();
        if path.exists() {
            self.settings = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
            self.settings.window_manager.enabled =
                self.settings.window_manager.enabled && is_virtual_desktop_supported();
        }
        Ok(())
    }

    fn load_weg_items(&mut self) -> Result<()> {
        let path = self.data_dir.join("seelenweg_items.yaml");
        self.weg_items = if !path.exists() {
            serde_yaml::Value::Null
        } else {
            serde_yaml::from_str(&std::fs::read_to_string(&path)?)?
        };
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

    fn load_themes(&mut self) -> Result<()> {
        let user_path = self.data_dir.join("themes");
        let resources_path = self.resources_dir.join("static/themes");
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

    fn load_placeholder_from_file(path: PathBuf) -> Result<Placeholder> {
        match path.extension() {
            Some(ext) if ext == "yml" || ext == "yaml" => {
                Ok(serde_yaml::from_str(&std::fs::read_to_string(&path)?)?)
            }
            _ => Err("Invalid placeholder file extension".into()),
        }
    }

    fn load_placeholders(&mut self) -> Result<()> {
        let user_path = self.data_dir.join("placeholders");
        let resources_path = self.resources_dir.join("static/placeholders");
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

    fn load_settings_by_app(&mut self) -> Result<()> {
        let user_apps_path = self.data_dir.join("applications.yml");
        let apps_templates_path = self.resources_dir.join("static/apps_templates");

        trace_lock!(REGEX_IDENTIFIERS).clear();
        self.settings_by_app.clear();

        if user_apps_path.exists() {
            let content = std::fs::read_to_string(&user_apps_path)?;
            let apps: Vec<AppConfig> = serde_yaml::from_str(&content)?;
            self.settings_by_app.extend(apps);
        }

        for entry in apps_templates_path.read_dir()?.flatten() {
            let content = std::fs::read_to_string(entry.path())?;
            let mut apps: Vec<AppConfig> = serde_yaml::from_str(&content)?;
            for app in apps.iter_mut() {
                app.is_bundled = true;
            }
            self.settings_by_app.extend(apps);
        }

        self.settings_by_app
            .iter()
            .for_each(|app| app.identifier.cache_regex());
        Ok(())
    }

    fn load_all(&mut self) -> Result<()> {
        self.load_settings()?;
        self.load_weg_items()?;
        self.load_themes()?;
        self.load_placeholders()?;
        self.load_settings_by_app()?;
        Ok(())
    }

    fn emit_settings(&self) -> Result<()> {
        self.handle.emit("settings", self.settings())?;
        trace_lock!(SEELEN).on_state_changed()?;
        Ok(())
    }

    fn emit_weg_items(&self) -> Result<()> {
        self.handle.emit("weg-items", self.weg_items())?;
        Ok(())
    }

    fn emit_themes(&self) -> Result<()> {
        self.handle
            .emit("themes", self.themes().values().collect_vec())?;
        Ok(())
    }

    fn emit_placeholders(&self) -> Result<()> {
        self.handle
            .emit("placeholders", self.placeholders().values().collect_vec())?;
        Ok(())
    }

    fn emit_settings_by_app(&self) -> Result<()> {
        self.handle
            .emit("settings-by-app", self.settings_by_app())?;
        Ok(())
    }

    pub fn save_settings(&self) -> Result<()> {
        std::fs::write(self.settings_path(), serde_json::to_string(&self.settings)?)?;
        Ok(())
    }

    async fn set_wallpaper(url: &str, path: &Path) -> Result<()> {
        let response = tauri_plugin_http::reqwest::get(url).await?;
        let contents = response.bytes().await?;
        std::fs::write(path, &contents)?;
        WindowsApi::set_wallpaper(path.to_string_lossy().to_string())?;
        Ok(())
    }

    pub fn load_resource(&mut self, resource: Resource) -> Result<()> {
        log::trace!("Loading resource: {}", resource.id);
        let id = resource.id;

        if let Some(image_url) = resource.wallpaper {
            let path = self.data_dir.join(format!("wallpapers/{id}.png"));
            tauri::async_runtime::spawn(async move {
                log_error!(Self::set_wallpaper(&image_url, &path).await);
            });
        }

        if let Some(theme) = resource.resources.theme {
            let filename = format!("{id}.yml");
            std::fs::write(
                self.data_dir.join(format!("themes/{filename}")),
                serde_yaml::to_string(&theme)?,
            )?;
            if !self.settings.selected_theme.contains(&filename) {
                self.settings.selected_theme.push(filename);
            }
        }

        if let Some(placeholder) = resource.resources.placeholder {
            std::fs::write(
                self.data_dir.join(format!("placeholders/{id}.yml")),
                serde_yaml::to_string(&placeholder)?,
            )?;
            self.settings.fancy_toolbar.placeholder = Some(format!("{id}.yml"));
        }

        if let Some(layout) = resource.resources.layout {
            std::fs::write(
                self.data_dir.join(format!("layouts/{id}.yml")),
                serde_yaml::to_string(&layout)?,
            )?;
            self.settings.window_manager.default_layout = Some(format!("{id}.yml"));
        }

        self.save_settings()?;
        Ok(())
    }
}
