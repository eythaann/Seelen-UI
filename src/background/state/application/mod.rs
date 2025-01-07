mod apps_config;
mod events;
mod icons;
mod plugins;
mod profiles;
mod settings;
mod weg_items;
mod widgets;

use arc_swap::ArcSwap;
use getset::Getters;
use itertools::Itertools;
use lazy_static::lazy_static;
use notify_debouncer_full::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode, Watcher},
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use parking_lot::Mutex;
use seelen_core::state::{
    IconPack, Plugin, PluginId, Profile, WegItems, Widget, WidgetId, WindowManagerLayout,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::{
    error_handler::Result, log_error, modules::cli::domain::Resource, trace_lock,
    utils::constants::SEELEN_COMMON, windows_api::WindowsApi,
};

use super::domain::{AppConfig, Placeholder, Settings, Theme};

lazy_static! {
    pub static ref FULL_STATE: Arc<ArcSwap<FullState>> = Arc::new(ArcSwap::from_pointee({
        log::trace!("Creating new State Manager");
        FullState::new().expect("Failed to create State Manager")
    }));
    static ref MODIFICATIONS_TO_SKIP: Arc<Mutex<HashSet<PathBuf>>> =
        Arc::new(Mutex::new(HashSet::new()));
}

pub type LauncherHistory = HashMap<String, Vec<String>>;

#[derive(Getters, Debug, Clone)]
#[getset(get = "pub")]
pub struct FullState {
    watcher: Arc<Option<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>>,
    // ======== data ========
    pub profiles: Vec<Profile>,
    pub settings: Settings,
    pub settings_by_app: VecDeque<AppConfig>,
    pub themes: HashMap<String, Theme>,
    pub icon_packs: Arc<Mutex<HashMap<String, IconPack>>>,
    pub placeholders: HashMap<String, Placeholder>,
    pub layouts: HashMap<String, WindowManagerLayout>,
    pub weg_items: WegItems,
    pub launcher_history: LauncherHistory,

    pub plugins: HashMap<PluginId, Plugin>,
    pub widgets: HashMap<WidgetId, Widget>,
}

unsafe impl Sync for FullState {}

impl FullState {
    fn new() -> Result<Self> {
        let mut manager = Self {
            watcher: Arc::new(None),
            // ======== data ========
            profiles: Vec::new(),
            settings: Settings::default(),
            settings_by_app: VecDeque::new(),
            themes: HashMap::new(),
            icon_packs: Arc::new(Mutex::new(HashMap::new())),
            placeholders: HashMap::new(),
            layouts: HashMap::new(),
            weg_items: WegItems::default(),
            launcher_history: HashMap::new(),
            plugins: HashMap::new(),
            widgets: HashMap::new(),
        };
        manager.load_all()?;
        manager.start_listeners()?;
        Ok(manager)
    }

    /// Shorthand of `FullState::clone` on Arc reference
    ///
    /// Intended to be used with `ArcSwap::rcu` to mofify the state
    pub fn cloned(&self) -> Self {
        self.clone()
    }

    pub fn skip_modification(&self, path: PathBuf) {
        trace_lock!(MODIFICATIONS_TO_SKIP).insert(path);
    }

    fn join_and_filter_debounced_changes(events: Vec<DebouncedEvent>) -> HashSet<PathBuf> {
        let mut result = HashSet::new();
        let mut to_skip = trace_lock!(MODIFICATIONS_TO_SKIP);
        for event in events {
            for path in event.event.paths {
                if !path.is_dir() {
                    result.insert(path);
                }
            }
        }

        let final_result = result.difference(&to_skip).cloned().collect();
        *to_skip = to_skip.difference(&result).cloned().collect();

        final_result
    }

    fn process_changes(&mut self, changed: &HashSet<PathBuf>) -> Result<()> {
        if changed
            .iter()
            .any(|p| p.starts_with(SEELEN_COMMON.icons_path()) && p.ends_with("metadata.yml"))
        {
            log::info!("Icons Packs changed");
            self.load_icons_packs()?;
            self.emit_icon_packs()?;
        }

        if changed.iter().any(|p| p == SEELEN_COMMON.weg_items_path()) {
            let old = self.weg_items.clone();
            self.read_weg_items()?;
            if old != self.weg_items {
                log::info!("Weg Items changed");
                self.emit_weg_items()?;
            }
        }

        if changed.iter().any(|p| p == SEELEN_COMMON.history_path()) {
            log::info!("History changed");
            self.load_history()?;
            self.emit_history()?;
        }

        if changed.iter().any(|p| p == SEELEN_COMMON.settings_path()) {
            log::info!("Seelen Settings changed");
            self.read_settings()?;
            self.emit_settings()?;
        }

        if changed.iter().any(|p| {
            p.starts_with(SEELEN_COMMON.user_themes_path())
                || p.starts_with(SEELEN_COMMON.bundled_themes_path())
        }) {
            log::info!("Theme changed");
            self.load_themes()?;
            self.emit_themes()?;
        }

        if changed.iter().any(|p| {
            p.starts_with(SEELEN_COMMON.user_placeholders_path())
                || p.starts_with(SEELEN_COMMON.bundled_placeholders_path())
        }) {
            log::info!("Placeholder changed");
            self.load_placeholders()?;
            self.emit_placeholders()?;
        }

        if changed.iter().any(|p| {
            p.starts_with(SEELEN_COMMON.user_layouts_path())
                || p.starts_with(SEELEN_COMMON.bundled_layouts_path())
        }) {
            log::info!("Layouts changed");
            self.load_layouts()?;
            self.emit_layouts()?;
        }

        if changed.iter().any(|p| {
            p == SEELEN_COMMON.user_app_configs_path()
                || p.starts_with(SEELEN_COMMON.bundled_app_configs_path())
        }) {
            log::info!("Specific App Configuration changed");
            self.load_settings_by_app()?;
            self.emit_settings_by_app()?;
        }

        if changed.iter().any(|p| {
            p.starts_with(SEELEN_COMMON.user_plugins_path())
                || p.starts_with(SEELEN_COMMON.bundled_plugins_path())
        }) {
            log::info!("Plugins changed");
            self.load_plugins()?;
            self.emit_plugins()?;
        }

        if changed.iter().any(|p| {
            p.starts_with(SEELEN_COMMON.user_widgets_path())
                || p.starts_with(SEELEN_COMMON.bundled_widgets_path())
        }) {
            log::info!("Widgets changed");
            self.load_widgets()?;
            self.emit_widgets()?;
        }

        Ok(())
    }

    fn start_listeners(&mut self) -> Result<()> {
        log::trace!("Starting Seelen UI Files Watcher");
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            |result: DebounceEventResult| match result {
                Ok(events) => {
                    // log::info!("Seelen UI File Watcher events: {:?}", events);
                    let changed = Self::join_and_filter_debounced_changes(events);
                    FULL_STATE.rcu(move |state| {
                        let mut state = state.cloned();
                        log_error!(state.process_changes(&changed));
                        state
                    });
                }
                Err(errors) => errors
                    .iter()
                    .for_each(|e| log::error!("File Watcher Error: {:?}", e)),
            },
        )?;

        let paths: Vec<&Path> = vec![
            // user data
            SEELEN_COMMON.settings_path(),
            SEELEN_COMMON.weg_items_path(),
            SEELEN_COMMON.user_app_configs_path(),
            SEELEN_COMMON.history_path(),
            SEELEN_COMMON.icons_path(),
            SEELEN_COMMON.user_themes_path(),
            SEELEN_COMMON.user_placeholders_path(),
            SEELEN_COMMON.user_layouts_path(),
            SEELEN_COMMON.user_plugins_path(),
            SEELEN_COMMON.user_widgets_path(),
            // bundled data
            SEELEN_COMMON.bundled_themes_path(),
            SEELEN_COMMON.bundled_placeholders_path(),
            SEELEN_COMMON.bundled_layouts_path(),
            SEELEN_COMMON.bundled_plugins_path(),
            SEELEN_COMMON.bundled_widgets_path(),
        ];

        for path in paths {
            debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
        }

        self.watcher = Arc::new(Some(debouncer));
        Ok(())
    }

    pub fn get_settings_from_path(path: &Path) -> Result<Settings> {
        match path.extension() {
            Some(ext) if ext == "json" => {
                Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
            }
            _ => Err("Invalid settings file extension".into()),
        }
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
            theme.styles.insert(
                WidgetId("weg".into()),
                std::fs::read_to_string(path.join("theme.weg.css"))?,
            );
        }

        if path.join("theme.toolbar.css").exists() {
            theme.styles.insert(
                WidgetId("toolbar".into()),
                std::fs::read_to_string(path.join("theme.toolbar.css"))?,
            );
        }

        if path.join("theme.wm.css").exists() {
            theme.styles.insert(
                WidgetId("wm".into()),
                std::fs::read_to_string(path.join("theme.wm.css"))?,
            );
        }

        if path.join("theme.launcher.css").exists() {
            theme.styles.insert(
                WidgetId("launcher".into()),
                std::fs::read_to_string(path.join("theme.launcher.css"))?,
            );
        }

        if path.join("theme.wall.css").exists() {
            theme.styles.insert(
                WidgetId("wall".into()),
                std::fs::read_to_string(path.join("theme.wall.css"))?,
            );
        }

        Ok(theme)
    }

    fn load_themes(&mut self) -> Result<()> {
        let entries = std::fs::read_dir(SEELEN_COMMON.bundled_themes_path())?
            .chain(std::fs::read_dir(SEELEN_COMMON.user_themes_path())?);
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
        let entries = std::fs::read_dir(SEELEN_COMMON.bundled_placeholders_path())?
            .chain(std::fs::read_dir(SEELEN_COMMON.user_placeholders_path())?);
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let placeholder = Self::load_placeholder_from_file(path);

            match placeholder {
                Ok(mut placeholder) => {
                    placeholder.sanitize();
                    placeholder.info.filename = entry.file_name().to_string_lossy().to_string();
                    self.placeholders
                        .insert(placeholder.info.filename.clone(), placeholder);
                }
                Err(err) => {
                    log::error!("Failed to load placeholder ({:?}): {:?}", entry.path(), err)
                }
            }
        }

        let selected = &mut self.settings.fancy_toolbar.placeholder;
        if !self.placeholders.contains_key(selected) {
            *selected = "default.yml".to_string();
        }

        Ok(())
    }

    fn load_layout_from_file(path: PathBuf) -> Result<WindowManagerLayout> {
        match path.extension() {
            Some(ext) if ext == "yml" || ext == "yaml" || ext == "json" => {
                let content = std::fs::read_to_string(&path)?;
                if ext == "json" {
                    Ok(serde_json::from_str(&content)?)
                } else {
                    Ok(serde_yaml::from_str(&content)?)
                }
            }
            _ => Err("Invalid layout file extension".into()),
        }
    }

    fn load_layouts(&mut self) -> Result<()> {
        let user_path = SEELEN_COMMON.user_layouts_path();
        let resources_path = SEELEN_COMMON.bundled_layouts_path();
        let entries = std::fs::read_dir(resources_path)?.chain(std::fs::read_dir(user_path)?);
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let layout = Self::load_layout_from_file(path);

            match layout {
                Ok(mut layout) => {
                    layout.info.filename = entry.file_name().to_string_lossy().to_string();
                    self.layouts.insert(layout.info.filename.clone(), layout);
                }
                Err(err) => {
                    log::error!("Failed to load layout ({:?}): {:?}", entry.path(), err)
                }
            }
        }

        let selected = &mut self.settings.window_manager.default_layout;
        if !self.layouts.contains_key(selected) {
            *selected = "BSP.json".to_string();
        }

        Ok(())
    }

    fn save_settings_by_app(&self) -> Result<()> {
        let data = self
            .settings_by_app
            .iter()
            .filter(|app| !app.is_bundled)
            .cloned()
            .collect_vec();
        std::fs::write(
            SEELEN_COMMON.user_app_configs_path(),
            serde_yaml::to_string(&data)?,
        )?;
        Ok(())
    }

    fn load_settings_by_app(&mut self) -> Result<()> {
        let user_apps_path = SEELEN_COMMON.user_app_configs_path();
        let apps_templates_path = SEELEN_COMMON.bundled_app_configs_path();

        self.settings_by_app.clear();
        if !user_apps_path.exists() {
            // save empty array on appdata dir
            self.save_settings_by_app()?;
        }

        for entry in apps_templates_path.read_dir()?.flatten() {
            let content = std::fs::read_to_string(entry.path())?;
            let mut apps: Vec<AppConfig> = serde_yaml::from_str(&content)?;
            for app in apps.iter_mut() {
                app.is_bundled = true;
            }
            self.settings_by_app.extend(apps);
        }

        if user_apps_path.exists() {
            let content = std::fs::read_to_string(user_apps_path)?;
            let apps: Vec<AppConfig> = serde_yaml::from_str(&content)?;
            self.settings_by_app.extend(apps);
        }

        self.settings_by_app
            .iter_mut()
            .for_each(|app| app.identifier.cache_regex());
        Ok(())
    }

    fn load_history(&mut self) -> Result<()> {
        let history_path = SEELEN_COMMON.history_path();
        if history_path.exists() {
            self.launcher_history = serde_yaml::from_str(&std::fs::read_to_string(history_path)?)?;
        } else {
            std::fs::write(history_path, serde_yaml::to_string(&self.launcher_history)?)?;
        }
        Ok(())
    }

    fn load_all(&mut self) -> Result<()> {
        self.read_settings()?;
        self.read_weg_items()?;
        self.load_themes()?;
        self.load_icons_packs()?;
        self.load_placeholders()?;
        self.load_layouts()?;
        self.load_settings_by_app()?;
        self.load_history()?;
        self.load_plugins()?;
        self.load_widgets()?;
        self.load_profiles()?;
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
            let path = SEELEN_COMMON.wallpapers_path().join(format!("{id}.png"));
            tauri::async_runtime::spawn(async move {
                log_error!(Self::set_wallpaper(&image_url, &path).await);
            });
        }

        let filename = format!("{id}.yml");
        if let Some(theme) = resource.resources.theme {
            std::fs::write(
                SEELEN_COMMON.user_themes_path().join(&filename),
                serde_yaml::to_string(&theme)?,
            )?;
            if !self.settings.selected_themes.contains(&filename) {
                self.settings.selected_themes.push(filename.clone());
            }
        }

        if let Some(placeholder) = resource.resources.placeholder {
            std::fs::write(
                SEELEN_COMMON.user_placeholders_path().join(&filename),
                serde_yaml::to_string(&placeholder)?,
            )?;
            self.settings.fancy_toolbar.placeholder = filename.clone();
        }

        if let Some(layout) = resource.resources.layout {
            std::fs::write(
                SEELEN_COMMON.user_layouts_path().join(&filename),
                serde_yaml::to_string(&layout)?,
            )?;
            self.settings.window_manager.default_layout = filename.clone();
        }

        self.write_settings()?;
        Ok(())
    }
}
