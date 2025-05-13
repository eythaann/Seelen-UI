mod apps_config;
mod events;
mod icons;
mod plugins;
mod profiles;
mod settings;
mod themes;
mod toolbar_items;
mod weg_items;
mod widgets;

use arc_swap::ArcSwap;
use getset::Getters;
use icons::IconPacksManager;
use itertools::Itertools;
use lazy_static::lazy_static;
use notify_debouncer_full::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode, Watcher},
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use parking_lot::Mutex;
use seelen_core::state::{LauncherHistory, Plugin, PluginId, Profile, WegItems, Widget, WidgetId};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::{error_handler::Result, log_error, utils::constants::SEELEN_COMMON};

use super::domain::{AppConfig, Placeholder, Settings, Theme};

lazy_static! {
    pub static ref FULL_STATE: Arc<ArcSwap<FullState>> = Arc::new(ArcSwap::from_pointee({
        log::trace!("Creating new State Manager");
        FullState::new().expect("Failed to create State Manager")
    }));
}

#[derive(Getters, Debug, Clone)]
#[getset(get = "pub")]
pub struct FullState {
    watcher: Arc<Option<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>>,
    // ======== data ========
    pub profiles: Vec<Profile>,
    pub settings: Settings,
    pub settings_by_app: VecDeque<AppConfig>,
    pub themes: HashMap<String, Theme>,
    pub icon_packs: Arc<Mutex<IconPacksManager>>,
    pub weg_items: WegItems,
    pub toolbar_items: Placeholder,
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
            icon_packs: Arc::new(Mutex::new(IconPacksManager::default())),
            weg_items: WegItems::default(),
            toolbar_items: Placeholder::default(),
            launcher_history: LauncherHistory::default(),
            plugins: HashMap::new(),
            widgets: HashMap::new(),
        };
        manager.load_all()?; // ScaDaned log shows a deadlock here.
        manager.start_listeners()?;
        Ok(manager)
    }

    /// Shorthand of `FullState::clone` on Arc reference
    ///
    /// Intended to be used with `ArcSwap::rcu` to mofify the state
    pub fn cloned(&self) -> Self {
        self.clone()
    }

    fn join_and_filter_debounced_changes(events: Vec<DebouncedEvent>) -> HashSet<PathBuf> {
        let mut result = HashSet::new();
        for event in events {
            for path in event.event.paths {
                if !path.is_dir() {
                    result.insert(path);
                }
            }
        }
        result
    }

    fn process_changes(&mut self, changed: &HashSet<PathBuf>) -> Result<()> {
        let mut is_changing_icons_metadata = false;
        let mut is_only_changing_system_icons = true;

        for path in changed.iter() {
            if path.starts_with(SEELEN_COMMON.user_icons_path()) && path.ends_with("metadata.yml") {
                is_changing_icons_metadata = true;
                if !path.ends_with("system\\metadata.yml") {
                    is_only_changing_system_icons = false;
                }
            }
        }

        if is_changing_icons_metadata {
            if !is_only_changing_system_icons {
                log::info!("Icons Packs changed");
                self.load_icons_packs(false)?;
            }
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

        if changed
            .iter()
            .any(|p| p == SEELEN_COMMON.toolbar_items_path())
        {
            let old = self.toolbar_items.clone();
            self.read_toolbar_items()?;
            if old != self.toolbar_items {
                log::info!("Toolbar Items changed");
                self.emit_toolbar_items()?;
            }
        }

        if changed.iter().any(|p| p == SEELEN_COMMON.history_path()) {
            log::info!("History changed");
            self.load_history()?;
            self.emit_history()?;
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

        // important: settings changed should be the last one to avoid use unexisting state
        // like new recently added theme, plugin, widget, etc
        if changed.iter().any(|p| p == SEELEN_COMMON.settings_path()) {
            log::info!("Seelen Settings changed");
            self.read_settings()?;
            self.emit_settings()?;
        }

        Ok(())
    }

    fn start_listeners(&mut self) -> Result<()> {
        log::trace!("Starting Seelen UI Files Watcher");
        let mut debouncer = new_debouncer(
            Duration::from_millis(100),
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
            SEELEN_COMMON.toolbar_items_path(),
            SEELEN_COMMON.user_app_configs_path(),
            SEELEN_COMMON.history_path(),
            SEELEN_COMMON.user_icons_path(),
            SEELEN_COMMON.user_themes_path(),
            SEELEN_COMMON.user_plugins_path(),
            SEELEN_COMMON.user_widgets_path(),
            // bundled data
            SEELEN_COMMON.bundled_themes_path(),
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
            .for_each(|app| app.identifier.perform_cache());
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

    /// We log each step on this cuz for some reason a deadlock is happening somewhere.
    fn load_all(&mut self) -> Result<()> {
        log::trace!("Initial load: settings");
        self.read_settings()?;

        log::trace!("Initial load: weg items");
        self.read_weg_items()?;

        log::trace!("Initial load: toolbar items");
        self.read_toolbar_items()?;

        log::trace!("Initial load: themes");
        self.load_themes()?;

        log::trace!("Initial load: icons packs");
        self.load_icons_packs(true)?;

        log::trace!("Initial load: plugins");
        self.load_settings_by_app()?;

        log::trace!("Initial load: history");
        self.load_history()?;

        log::trace!("Initial load: plugins");
        self.load_plugins()?;

        log::trace!("Initial load: widgets");
        self.load_widgets()?;

        log::trace!("Initial load: profiles");
        self.load_profiles()?;
        Ok(())
    }
}
