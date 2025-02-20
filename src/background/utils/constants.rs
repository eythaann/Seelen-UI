use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use itertools::Itertools;
use lazy_static::lazy_static;
use tauri::Manager;

use crate::seelen::get_app_handle;

lazy_static! {
    pub static ref SEELEN_COMMON: Arc<SeelenCommon> = Arc::new(SeelenCommon::new());

    /**
     * Some UWP apps like WhatsApp are resized after be opened,
     * this list will be used to resize them back after a delay.
     */
    pub static ref FORCE_RETILING_AFTER_ADD: Vec<String> = ["WhatsApp"]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();
}

pub static NATIVE_UI_POPUP_CLASSES: [&str; 3] = [
    "ForegroundStaging",            // Task Switching and Task View
    "XamlExplorerHostIslandWindow", // Task Switching, Task View and other popups
    "ControlCenterWindow",          // Windows 11 right panel with quick settings
];

pub static OVERLAP_BLACK_LIST_BY_EXE: [&str; 6] = [
    "msedgewebview2.exe",
    "SearchHost.exe",
    "StartMenuExperienceHost.exe",
    "ShellExperienceHost.exe",
    "GameBar.exe",      // Windows Xbox Game Bar
    "SnippingTool.exe", // Windows Snipping Tool
];

pub struct SeelenCommon {
    // general
    resource_dir: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
    temp_dir: PathBuf,
    // specifits
    history: PathBuf,
    settings: PathBuf,
    weg_items: PathBuf,
    toolbar_items: PathBuf,
    icons: PathBuf,
    user_themes: PathBuf,
    bundled_themes: PathBuf,
    user_plugins: PathBuf,
    bundled_plugins: PathBuf,
    user_app_configs: PathBuf,
    bundled_app_configs: PathBuf,
    /// @deprecated since v2.1.0
    user_placeholders: PathBuf,
    widgets: PathBuf,
    bundled_widgets: PathBuf,
    wallpapers: PathBuf,
    profiles: PathBuf,
    bundled_profiles: PathBuf,
}

impl SeelenCommon {
    pub fn new() -> Self {
        let resolver = get_app_handle().path();
        let data_dir = resolver.app_data_dir().expect("Failed to get app data dir");
        let resource_dir = resolver.resource_dir().expect("Failed to get resource dir");
        let cache_dir = resolver.app_cache_dir().expect("Failed to get cache dir");
        let temp_dir = resolver
            .temp_dir()
            .expect("Failed to get temp dir")
            .join("com.seelen.seelen-ui");

        Self {
            history: data_dir.join("history"),
            settings: data_dir.join("settings.json"),
            weg_items: data_dir.join("seelenweg_items_v2.yml"),
            toolbar_items: data_dir.join("toolbar_items.yml"),
            icons: data_dir.join("icons"),
            user_themes: data_dir.join("themes"),
            bundled_themes: resource_dir.join("static/themes"),
            user_plugins: data_dir.join("plugins"),
            bundled_plugins: resource_dir.join("static/plugins"),
            user_app_configs: data_dir.join("applications.yml"),
            bundled_app_configs: resource_dir.join("static/apps_templates"),
            user_placeholders: data_dir.join("placeholders"),
            widgets: data_dir.join("widgets"),
            bundled_widgets: resource_dir.join("static/widgets"),
            wallpapers: data_dir.join("wallpapers"),
            profiles: data_dir.join("profiles"),
            bundled_profiles: resource_dir.join("static/profiles"),
            // general
            data_dir,
            resource_dir,
            cache_dir,
            temp_dir,
        }
    }

    pub fn app_resource_dir(&self) -> &Path {
        &self.resource_dir
    }

    pub fn app_data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn app_cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn app_temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    pub fn settings_path(&self) -> &Path {
        &self.settings
    }

    pub fn weg_items_path(&self) -> &Path {
        &self.weg_items
    }

    pub fn toolbar_items_path(&self) -> &Path {
        &self.toolbar_items
    }

    pub fn history_path(&self) -> &Path {
        &self.history
    }

    pub fn icons_path(&self) -> &Path {
        &self.icons
    }

    pub fn user_themes_path(&self) -> &Path {
        &self.user_themes
    }

    pub fn bundled_themes_path(&self) -> &Path {
        &self.bundled_themes
    }

    pub fn user_plugins_path(&self) -> &Path {
        &self.user_plugins
    }

    pub fn bundled_plugins_path(&self) -> &Path {
        &self.bundled_plugins
    }

    pub fn user_app_configs_path(&self) -> &Path {
        &self.user_app_configs
    }

    pub fn bundled_app_configs_path(&self) -> &Path {
        &self.bundled_app_configs
    }

    /// @deprecated since v2.1.0 will be removed in v3.0
    pub fn user_placeholders_path(&self) -> &Path {
        &self.user_placeholders
    }

    pub fn user_widgets_path(&self) -> &Path {
        &self.widgets
    }

    pub fn bundled_widgets_path(&self) -> &Path {
        &self.bundled_widgets
    }

    pub fn wallpapers_path(&self) -> &Path {
        &self.wallpapers
    }

    pub fn user_profiles_path(&self) -> &Path {
        &self.profiles
    }

    pub fn bundled_profiles_path(&self) -> &Path {
        &self.bundled_profiles
    }
}
