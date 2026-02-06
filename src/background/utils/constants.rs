use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use seelen_core::resource::{ResourceId, WidgetId};
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
    icons: PathBuf,
    system_icon_pack: PathBuf,
    user_themes: PathBuf,
    bundled_themes: PathBuf,
    user_plugins: PathBuf,
    bundled_plugins: PathBuf,
    bundled_app_configs: PathBuf,
    wallpapers: PathBuf,
    widgets: PathBuf,
    bundled_widgets: PathBuf,
    sounds: PathBuf,
    // system
    system_dir: PathBuf,
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
            icons: data_dir.join("iconpacks"),
            system_icon_pack: cache_dir.join("gen-icon-pack"),
            sounds: data_dir.join("soundpacks"),
            user_themes: data_dir.join("themes"),
            bundled_themes: resource_dir.join("static/themes"),
            user_plugins: data_dir.join("plugins"),
            bundled_plugins: resource_dir.join("static/plugins"),
            bundled_app_configs: resource_dir.join("static/apps_templates"),
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

    pub fn bundled_app_configs_path(&self) -> &Path {
        &self.bundled_app_configs
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

    pub fn widget_data_dir(&self, id: &WidgetId) -> PathBuf {
        let data_dir = SEELEN_COMMON.app_data_dir().join("data");
        let folder = match &**id {
            ResourceId::Local(id) => id.trim_start_matches("@").replace("/", "-"),
            ResourceId::Remote(uuid) => uuid.to_string(),
        };
        data_dir.join(folder)
    }
}
