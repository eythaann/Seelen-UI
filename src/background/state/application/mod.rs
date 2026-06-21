mod apps_config;
pub mod performance;
mod settings;
mod toolbar_items;
mod weg_items;

pub use toolbar_items::TOOLBAR_ITEMS_MANAGER;
pub use weg_items::WEG_ITEMS_MANAGER;

use arc_swap::ArcSwap;
use notify_debouncer_full::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode, Watcher},
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use seelen_core::{
    resource::ResourceKind,
    state::{AppsConfigurationList, Settings},
};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, LazyLock, OnceLock},
    time::Duration,
};

use crate::{
    error::{Result, ResultLogExt},
    resources::{ResourceManager, RESOURCES},
    utils::constants::SEELEN_COMMON,
};

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

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub settings: Settings,
}

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
}

static USER_RESOURCES_WATCHER: OnceLock<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>> =
    OnceLock::new();

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

    debouncer
        .watcher()
        .watch(SEELEN_COMMON.app_data_dir(), RecursiveMode::Recursive)?;
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
        RESOURCES.load_all_of_type(ResourceKind::Widget).await?;
        RESOURCES.emit_widgets()?;
    }

    if themes_changed {
        log::info!("Themes changed");
        RESOURCES.load_all_of_type(ResourceKind::Theme).await?;
        RESOURCES.emit_themes();
    }

    if plugins_changed {
        log::info!("Plugins changed");
        RESOURCES.load_all_of_type(ResourceKind::Plugin).await?;
        RESOURCES.emit_plugins();
    }

    if wallpapers_changed {
        log::info!("Wallpapers changed");
        RESOURCES.load_all_of_type(ResourceKind::Wallpaper).await?;
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
        RESOURCES.load_all_of_type(ResourceKind::IconPack).await?;
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
