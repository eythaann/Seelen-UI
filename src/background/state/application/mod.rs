mod apps_config;
mod events;
mod icons;
pub mod performance;
mod settings;
mod toolbar_items;
mod weg_items;

pub use icons::download_remote_icons;

use arc_swap::ArcSwap;
use itertools::Itertools;
use lazy_static::lazy_static;
use notify_debouncer_full::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode, Watcher},
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use seelen_core::{
    resource::ResourceKind,
    state::{AppsConfigurationList, CssStyles, SluPopupConfig, SluPopupContent, WegItems},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::{
    error::Result, log_error, resources::RESOURCES, utils::constants::SEELEN_COMMON,
    widgets::popups::POPUPS_MANAGER,
};

use super::domain::{AppConfig, Placeholder, Settings};

lazy_static! {
    pub static ref FULL_STATE: Arc<ArcSwap<FullState>> = Arc::new(ArcSwap::from_pointee({
        log::trace!("Creating new State Manager");
        FullState::new().expect("Failed to create State Manager")
    }));
}

#[derive(Debug, Clone)]
pub struct FullState {
    watcher: Arc<Option<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>>,
    // ======== data ========
    pub settings: Settings,
    pub settings_by_app: AppsConfigurationList,
    pub weg_items: WegItems,
    pub toolbar_items: Placeholder,
}

unsafe impl Sync for FullState {}

impl FullState {
    fn new() -> Result<Self> {
        let mut manager = Self {
            watcher: Arc::new(None),
            // ======== data ========
            settings: Settings::default(),
            settings_by_app: AppsConfigurationList::default(),
            weg_items: WegItems::default(),
            toolbar_items: Self::initial_toolbar_items(),
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
        let mut widgets_changed = false;
        let mut icons_changed = false;
        let mut themes_changed = false;
        let mut plugins_changed = false;
        let mut wallpapers_changed = false;

        let mut settings_changed = false;
        let mut weg_items_changed = false;
        let mut toolbar_items_changed = false;
        let mut app_configs_changed = false;

        // Single iteration over the changed paths
        for path in changed {
            if !icons_changed && path.starts_with(SEELEN_COMMON.user_icons_path()) {
                icons_changed = true;
            };

            if !weg_items_changed && path == SEELEN_COMMON.weg_items_path() {
                weg_items_changed = true;
            }

            if !toolbar_items_changed && path == SEELEN_COMMON.toolbar_items_path() {
                toolbar_items_changed = true;
            }

            if !themes_changed
                && (path.starts_with(SEELEN_COMMON.user_themes_path())
                    || path.starts_with(SEELEN_COMMON.bundled_themes_path()))
            {
                themes_changed = true;
            }

            if !app_configs_changed
                && (path == SEELEN_COMMON.user_app_configs_path()
                    || path.starts_with(SEELEN_COMMON.bundled_app_configs_path()))
            {
                app_configs_changed = true;
            }

            if !plugins_changed
                && (path.starts_with(SEELEN_COMMON.user_plugins_path())
                    || path.starts_with(SEELEN_COMMON.bundled_plugins_path()))
            {
                plugins_changed = true;
            }

            if !widgets_changed
                && (path.starts_with(SEELEN_COMMON.user_widgets_path())
                    || path.starts_with(SEELEN_COMMON.bundled_widgets_path()))
            {
                widgets_changed = true;
            }

            if !settings_changed && path == SEELEN_COMMON.settings_path() {
                settings_changed = true;
            }

            if !wallpapers_changed && path.starts_with(SEELEN_COMMON.user_wallpapers_path()) {
                wallpapers_changed = true;
            }
        }

        if weg_items_changed {
            let old = self.weg_items.clone();
            self.read_weg_items();
            if old != self.weg_items {
                log::info!("Weg Items changed");
                self.emit_weg_items()?;
            }
        }

        if toolbar_items_changed {
            let old = self.toolbar_items.clone();
            self.read_toolbar_items();
            if old != self.toolbar_items {
                log::info!("Toolbar Items changed");
                self.emit_toolbar_items()?;
            }
        }

        if app_configs_changed {
            log::info!("Specific App Configuration changed");
            self.load_settings_by_app();
            self.emit_settings_by_app()?;
        }

        if themes_changed {
            log::info!("Theme changed");
            RESOURCES.load_all_of_type(ResourceKind::Theme)?;
            RESOURCES.emit_themes()?;
        }

        if icons_changed {
            log::info!("Icon Packs changed");
            RESOURCES.load_all_of_type(ResourceKind::IconPack)?;
            RESOURCES.emit_icon_packs()?;
        }

        if plugins_changed {
            log::info!("Plugins changed");
            RESOURCES.load_all_of_type(ResourceKind::Plugin)?;
            RESOURCES.emit_plugins()?;
        }

        if widgets_changed {
            log::info!("Widgets changed");
            RESOURCES.load_all_of_type(ResourceKind::Widget)?;
            RESOURCES.emit_widgets()?;
        }

        if wallpapers_changed {
            log::info!("Wallpapers changed");
            RESOURCES.load_all_of_type(ResourceKind::Wallpaper)?;
            RESOURCES.emit_wallpapers()?;

            if self.sanitize_wallpaper_collections() {
                self.emit_settings()?;
            }
        }

        // important: settings changed should be the last one to avoid use unexisting state
        // like new recently added theme, plugin, widget, etc
        if settings_changed {
            log::info!("Seelen Settings changed");
            self.read_settings();
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
                    .for_each(|e| log::error!("File Watcher Error: {e:?}")),
            },
        )?;

        let paths: Vec<&Path> = vec![
            // user data
            SEELEN_COMMON.settings_path(),
            SEELEN_COMMON.weg_items_path(),
            SEELEN_COMMON.toolbar_items_path(),
            SEELEN_COMMON.user_app_configs_path(),
            SEELEN_COMMON.user_icons_path(),
            SEELEN_COMMON.user_themes_path(),
            SEELEN_COMMON.user_plugins_path(),
            SEELEN_COMMON.user_widgets_path(),
            SEELEN_COMMON.user_wallpapers_path(),
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

    fn _load_settings_by_app(&mut self) -> Result<()> {
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

        self.settings_by_app.prepare();
        Ok(())
    }

    fn load_settings_by_app(&mut self) {
        if let Err(e) = self._load_settings_by_app() {
            log::error!("Error loading settings by app: {e}");
            Self::show_corrupted_state_to_user(SEELEN_COMMON.user_app_configs_path());
        }
    }

    /// We log each step on this cuz for some reason a deadlock is happening somewhere.
    fn load_all(&mut self) -> Result<()> {
        log::trace!("Initial load: settings");
        self.read_settings();

        log::trace!("Initial load: weg items");
        self.read_weg_items();

        log::trace!("Initial load: toolbar items");
        self.read_toolbar_items();

        log::trace!("Initial load: settings by app");
        self.load_settings_by_app();
        Ok(())
    }

    fn show_corrupted_state_to_user(path: &Path) {
        let mut manager = POPUPS_MANAGER.lock();
        let config = SluPopupConfig {
            title: vec![SluPopupContent::Group {
                items: vec![
                    SluPopupContent::Icon {
                        name: "BiSolidError".to_string(),
                        styles: Some(
                            CssStyles::new()
                                .add("color", "var(--color-red-800)")
                                .add("height", "1.2rem"),
                        ),
                    },
                    SluPopupContent::Text {
                        value: t!("runtime.corrupted_data").to_string(),
                        styles: None,
                    },
                ],
                styles: Some(CssStyles::new().add("alignItems", "center")),
            }],
            content: vec![
                SluPopupContent::Text {
                    value: t!("runtime.corrupted_file").to_string(),
                    styles: None,
                },
                SluPopupContent::Text {
                    value: format!("{}: {:?}", t!("runtime.corrupted_file_path"), path),
                    styles: None,
                },
            ],
            ..Default::default()
        };
        log_error!(manager.create(config));
    }
}
