mod apps_config;
mod performance;
mod settings;
mod toolbar_items;
mod weg_items;

pub use performance::PERFORMANCE_MODE;
pub use toolbar_items::TOOLBAR_ITEMS_MANAGER;
pub use weg_items::WEG_ITEMS_MANAGER;

use arc_swap::ArcSwap;
use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache, new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
};
use seelen_core::{
    resource::{PluginId, ResourceKind, WidgetId},
    state::{
        AppsConfigurationList, Settings, WorkspaceId,
        by_widget::GenericWidgetSettings,
        twm::TwmPlugin,
        value::{KnownPlugin, PluginValue},
    },
    system_state::MonitorId,
};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, LazyLock, OnceLock},
    time::Duration,
};
use uuid::Uuid;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    resources::{RESOURCES, ResourceManager},
    utils::constants::SEELEN_COMMON,
    virtual_desktops::VdManager,
};

use super::domain::StateEvent;

/// Key of the layout setting declared on the window manager widget metadata
const WM_LAYOUT_SETTING_KEY: &str = "layout";

pub static BUNDLED_SETTINGS_BY_APP: LazyLock<Arc<AppsConfigurationList>> = LazyLock::new(|| {
    Arc::new(match AppSettings::load_bundled_settings_by_app() {
        Ok(list) => list,
        Err(e) => {
            log::error!("Failed to load bundled settings by app: {e}");
            AppsConfigurationList::default()
        }
    })
});

pub static FULL_STATE: LazyLock<ArcSwap<AppSettings>> = LazyLock::new(|| {
    ArcSwap::from_pointee({
        log::trace!("Creating new State Manager");
        AppSettings::new()
    })
});

static USER_RESOURCES_WATCHER: OnceLock<Debouncer<RecommendedWatcher, RecommendedCache>> =
    OnceLock::new();

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub settings: Settings,
}

/// Emitter of the state events (`FULL_STATE` is swapped as a whole, so it can't own them).
pub struct StateManager;

event_manager!(StateManager, StateEvent);

impl AppSettings {
    fn new() -> Self {
        Self {
            settings: AppSettings::read_settings(),
        }
    }

    /// Shorthand of `FullState::clone` on Arc reference
    ///
    /// Intended to be used with `ArcSwap::rcu` to mofify the state
    pub fn cloned(&self) -> Self {
        self.clone()
    }

    /// Run RESOURCES-dependent initialization steps.
    /// Call this after both FULL_STATE and RESOURCES are initialized so the two can
    /// be loaded in parallel.
    pub fn complete_initialization(&mut self, resources: &ResourceManager) {
        self.migration_v2_5_0(resources).log_error();
        self.sanitize_wallpaper_collections(resources);
    }

    pub fn is_widget_enabled(&self, widget_id: &WidgetId) -> bool {
        self.settings.is_widget_enabled(widget_id)
    }

    pub fn is_widget_enable_on_monitor(
        &self,
        widget_id: &WidgetId,
        monitor_id: &MonitorId,
    ) -> bool {
        self.settings
            .is_widget_enabled_on_monitor(widget_id, monitor_id)
    }

    pub fn is_weg_enabled(&self) -> bool {
        self.is_widget_enabled(&WidgetId::known_weg())
    }

    pub fn get_widget_instances_ids(&self, widget_id: &WidgetId) -> Vec<Uuid> {
        let config = self.settings.by_widget.others.get(widget_id);
        match config {
            Some(config) => config
                .instances
                .as_ref()
                .map_or_else(Default::default, |i| i.keys().cloned().collect()),
            None => Vec::new(),
        }
    }

    pub fn get_wm_layout(&self, workspace_id: &WorkspaceId) -> TwmPlugin {
        let base = TwmPlugin::default();

        let layout_id = self.get_wm_layout_id(workspace_id);

        let mut plugin_with_layout = None;
        RESOURCES.plugins.any_sync(|_, p| {
            if p.id == layout_id {
                plugin_with_layout = Some(p.clone());
                true
            } else {
                false
            }
        });

        let Some(plugin) = plugin_with_layout else {
            return base;
        };

        let PluginValue::Known(plugin) = &plugin.plugin else {
            return base;
        };

        let KnownPlugin::WManager(layout) = &**plugin else {
            return base;
        };

        *layout.clone()
    }

    /// Priority: workspace layout → monitor layout → global layout → bspwm
    pub fn get_wm_layout_id(&self, workspace_id: &WorkspaceId) -> PluginId {
        let wm_id = WidgetId::known_wm();
        let from_overrides = |settings: Option<&GenericWidgetSettings>| -> Option<PluginId> {
            let value = settings?.rest.get(WM_LAYOUT_SETTING_KEY)?;
            let layout_id: PluginId = value.0.as_str()?.into();
            layout_id.is_valid().then_some(layout_id)
        };

        let monitor_config = VdManager::instance()
            .get_monitor_of_workspace(workspace_id)
            .and_then(|monitor_id| self.settings.monitors_v3.get(&monitor_id));

        if let Some(monitor_config) = monitor_config {
            let by_workspace = monitor_config
                .by_workspace
                .get(workspace_id)
                .and_then(|config| from_overrides(config.by_widget.get(&wm_id)));
            if let Some(layout_id) = by_workspace {
                return layout_id;
            }

            if let Some(layout_id) = from_overrides(monitor_config.by_widget.get(&wm_id)) {
                return layout_id;
            }
        }

        let default = self.settings.by_widget.wm.layout.clone();
        if default.is_valid() {
            default
        } else {
            "@default/wm-bspwm".into()
        }
    }

    pub fn locale(&self) -> &String {
        &self.settings.language
    }
}

pub fn initialize_user_resources_watcher() -> Result<()> {
    log::trace!("Starting Seelen UI Files Watcher");
    let mut debouncer = new_debouncer(
        Duration::from_millis(100),
        None,
        |result: DebounceEventResult| match result {
            Ok(events) => {
                let changed = join_and_filter_debounced_changes(events);
                crate::get_tokio_handle()
                    .spawn(async move { process_changes(&changed).await.log_error() });
            }
            Err(errors) => errors
                .iter()
                .for_each(|e| log::error!("File Watcher Error: {e:?}")),
        },
    )?;

    debouncer.watch(SEELEN_COMMON.app_data_dir(), RecursiveMode::Recursive)?;
    USER_RESOURCES_WATCHER.set(debouncer).ok();
    Ok(())
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

async fn process_changes(changed: &HashSet<PathBuf>) -> Result<()> {
    let mut widgets_changed = false;
    let mut icons_changed = false;
    let mut themes_changed = false;
    let mut plugins_changed = false;
    let mut wallpapers_changed = false;

    // Single iteration over the changed paths
    for path in changed {
        if !icons_changed && path.starts_with(SEELEN_COMMON.user_icons_path()) {
            icons_changed = true;
        };

        if !themes_changed
            && (path.starts_with(SEELEN_COMMON.user_themes_path())
                || path.starts_with(SEELEN_COMMON.bundled_themes_path()))
        {
            themes_changed = true;
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

        if !wallpapers_changed && path.starts_with(SEELEN_COMMON.user_wallpapers_path()) {
            wallpapers_changed = true;
        }
    }

    if widgets_changed {
        log::info!("Widgets changed");
        RESOURCES
            .load_all_of_type(ResourceKind::Widget, false)
            .await?;
        RESOURCES.emit_widgets()?;
    }

    if themes_changed {
        log::info!("Themes changed");
        RESOURCES
            .load_all_of_type(ResourceKind::Theme, false)
            .await?;
        RESOURCES.emit_themes();
    }

    if plugins_changed {
        log::info!("Plugins changed");
        RESOURCES
            .load_all_of_type(ResourceKind::Plugin, false)
            .await?;
        RESOURCES.emit_plugins();
    }

    if wallpapers_changed {
        log::info!("Wallpapers changed");
        RESOURCES
            .load_all_of_type(ResourceKind::Wallpaper, false)
            .await?;
        RESOURCES.emit_wallpapers();

        FULL_STATE.rcu(move |state| {
            let mut state = state.cloned();
            if state.sanitize_wallpaper_collections(&RESOURCES) {
                state.emit_settings().log_error();
            }
            state
        });
    }

    if icons_changed {
        log::info!("Icon Packs changed");
        RESOURCES
            .load_all_of_type(ResourceKind::IconPack, false)
            .await?;
        RESOURCES.emit_icon_packs();
    }

    // important: settings changed should be the last one to avoid use unexisting state
    // like new recently added theme, plugin, widget, etc
    /* if settings_changed {
        log::info!("Seelen Settings changed");
        self.read_settings();
        self.emit_settings()?;
    } */

    Ok(())
}
