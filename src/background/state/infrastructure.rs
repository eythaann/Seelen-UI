use std::path::PathBuf;

use seelen_core::state::{
    AppConfig, IconPackEntry, PerformanceMode, Settings, ToolbarState, Wallpaper, WegItems,
    by_monitor::MonitorConfiguration, by_wallpaper::WallpaperInstanceSettings,
};
use tauri_plugin_dialog::DialogExt;

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    state::application::{BUNDLED_SETTINGS_BY_APP, performance::PERFORMANCE_MODE},
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
    windows_api::WindowsApi,
};

use super::application::{FULL_STATE, TOOLBAR_ITEMS_MANAGER, WEG_ITEMS_MANAGER};

impl crate::tauri_handlers::Handlers {
    pub fn state_get_toolbar_items() -> ToolbarState {
        TOOLBAR_ITEMS_MANAGER.get()
    }

    pub fn state_write_toolbar_items(items: ToolbarState) -> Result<()> {
        TOOLBAR_ITEMS_MANAGER.write(items)
    }

    pub fn state_get_weg_items() -> WegItems {
        WEG_ITEMS_MANAGER.get()
    }

    pub fn state_write_weg_items(items: WegItems) -> Result<()> {
        WEG_ITEMS_MANAGER.write(items)
    }

    pub fn state_get_settings(path: Option<PathBuf>) -> Result<Settings> {
        if let Some(path) = path {
            Ok(Settings::load(path)?)
        } else {
            Ok(FULL_STATE.load().settings.clone())
        }
    }

    pub fn state_get_default_settings() -> Result<Settings> {
        let mut settings = Settings::default();
        settings.sanitize()?;
        Ok(settings)
    }

    pub fn state_get_default_monitor_settings() -> MonitorConfiguration {
        MonitorConfiguration::default()
    }

    pub fn state_get_default_wallpaper_settings() -> WallpaperInstanceSettings {
        WallpaperInstanceSettings::default()
    }

    pub fn state_write_settings(mut settings: Settings) -> Result<()> {
        settings.sanitize()?;
        FULL_STATE.rcu(move |state| {
            let mut state = state.cloned();
            state.settings = settings.clone();
            state
        });
        FULL_STATE.load().write_settings()?;
        crate::backups::application::on_settings_saved();
        Ok(())
    }

    pub fn state_get_settings_by_app() -> Vec<AppConfig> {
        BUNDLED_SETTINGS_BY_APP.iter().cloned().collect()
    }

    pub fn get_native_shell_wallpaper() -> Result<PathBuf> {
        WindowsApi::get_wallpaper()
    }

    pub fn set_native_shell_wallpaper(path: PathBuf) -> Result<()> {
        WindowsApi::set_wallpaper(path)
    }

    pub fn state_request_wallpaper_addition() -> Result<()> {
        get_app_handle()
            .dialog()
            .file()
            .set_title("Pick Wallpapers")
            .add_filter("video", &Wallpaper::SUPPORTED_VIDEOS)
            .add_filter("image", &Wallpaper::SUPPORTED_IMAGES)
            .pick_files(|picked| {
                for path in picked.unwrap_or_default() {
                    if let Ok(path) = path.simplified().into_path() {
                        let folder_to_store = SEELEN_COMMON
                            .user_wallpapers_path()
                            .join(date_based_hex_id());
                        crate::get_tokio_handle().spawn(async move {
                            Wallpaper::create_from_file(&path, &folder_to_store, true)
                                .await
                                .log_error();
                        });
                    }
                }
            });
        Ok(())
    }

    pub fn state_add_icon_to_custom_icon_pack(_icon: IconPackEntry) -> Result<()> {
        todo!()
    }

    pub fn state_get_performance_mode() -> PerformanceMode {
        PERFORMANCE_MODE.load()
    }
}
