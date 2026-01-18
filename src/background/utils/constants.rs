use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use tauri::Manager;
use windows::Win32::UI::Shell::FOLDERID_Windows;

use crate::{app::get_app_handle, windows_api::WindowsApi};

pub static SEELEN_COMMON: LazyLock<Arc<SeelenCommon>> =
    LazyLock::new(|| Arc::new(SeelenCommon::new()));

pub struct SeelenCommon {
    // general
    resource_dir: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
    temp_dir: PathBuf,
    // specifits
    settings: PathBuf,
    weg_items: PathBuf,
    toolbar_items: PathBuf,
    icons: PathBuf,
    system_icon_pack: PathBuf,
    user_themes: PathBuf,
    bundled_themes: PathBuf,
    user_plugins: PathBuf,
    bundled_plugins: PathBuf,
    user_app_configs: PathBuf,
    bundled_app_configs: PathBuf,
    wallpapers: PathBuf,
    widgets: PathBuf,
    bundled_widgets: PathBuf,
    sounds: PathBuf,
    // system
    system_dir: PathBuf,

    // @deprecated since v2.1.0
    user_placeholders: PathBuf,
}

#[allow(dead_code)]
impl SeelenCommon {
    pub fn new() -> Self {
        let resolver = get_app_handle().path();

        let resource_dir = resolver.resource_dir().expect("Failed to get resource dir");
        let data_dir = resolver.app_data_dir().expect("Failed to get app data dir");
        let cache_dir = resolver.app_cache_dir().expect("Failed to get cache dir");
        let temp_dir = resolver
            .temp_dir()
            .expect("Failed to get temp dir")
            .join("com.seelen.seelen-ui");

        let system_dir =
            WindowsApi::known_folder(FOLDERID_Windows).expect("Failed to get system dir");

        Self {
            settings: data_dir.join("settings.json"),
            weg_items: data_dir.join("seelenweg_items_v2.yml"),
            toolbar_items: data_dir.join("toolbar_items.yml"),
            icons: data_dir.join("iconpacks"),
            system_icon_pack: cache_dir.join("gen-icon-pack"),
            sounds: data_dir.join("soundpacks"),
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
            // general
            data_dir,
            resource_dir,
            cache_dir,
            temp_dir,
            system_dir,
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

    /// Windows: `X:\Windows`
    pub fn system_dir(&self) -> &Path {
        &self.system_dir
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

    pub fn system_icon_pack_path(&self) -> &Path {
        &self.system_icon_pack
    }

    pub fn user_icons_path(&self) -> &Path {
        &self.icons
    }

    pub fn user_sounds_path(&self) -> &Path {
        &self.sounds
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

    pub fn user_wallpapers_path(&self) -> &Path {
        &self.wallpapers
    }
}
