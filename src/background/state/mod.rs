//! Seelen UI user state: settings, per-widget items (toolbar, weg), performance mode and the
//! watcher of the user resources folder.

mod application;
mod domain;

use std::{path::PathBuf, sync::Once};

use seelen_core::{
    handlers::SeelenEvent,
    state::{
        AppConfig, IconPackEntry, PerformanceMode, Settings, ToolbarState, Wallpaper, WegItems,
        by_monitor::MonitorConfiguration, by_wallpaper::WallpaperInstanceSettings,
    },
};
use tauri::WebviewWindow;
use tauri_plugin_dialog::DialogExt;

use crate::{
    app::{emit_to_webviews, get_app_handle},
    error::{Result, ResultLogExt},
    platform::shell,
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
    widgets::permissions::{WidgetPerm, request_widget_permission},
};

use self::application::BUNDLED_SETTINGS_BY_APP;

pub use self::{
    application::{
        AppSettings, FULL_STATE, PERFORMANCE_MODE, StateManager, TOOLBAR_ITEMS_MANAGER,
        WEG_ITEMS_MANAGER, initialize_user_resources_watcher,
    },
    domain::StateEvent,
};

/// Bridges the state events to the webviews. Registered on the first command access, as the
/// webviews always fetch the state before listening to its changes.
fn register_webviews_bridge() {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        StateManager::subscribe(|event| match event {
            StateEvent::SettingsChanged { settings } => {
                emit_to_webviews(SeelenEvent::StateSettingsChanged, &*settings);
            }
            StateEvent::PerformanceModeChanged { mode } => {
                emit_to_webviews(SeelenEvent::StatePerformanceModeChanged, mode);
            }
        });
    });
}

impl crate::tauri_handlers::Handlers {
    pub fn state_get_toolbar_items() -> ToolbarState {
        TOOLBAR_ITEMS_MANAGER.get()
    }

    pub fn state_write_toolbar_items(webview: WebviewWindow, items: ToolbarState) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;
        TOOLBAR_ITEMS_MANAGER.write(items)
    }

    pub fn state_get_weg_items() -> WegItems {
        WEG_ITEMS_MANAGER.get()
    }

    pub fn state_write_weg_items(webview: WebviewWindow, items: WegItems) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;
        WEG_ITEMS_MANAGER.write(items)
    }

    pub fn state_get_settings(path: Option<PathBuf>) -> Result<Settings> {
        register_webviews_bridge();
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

    pub fn state_write_settings(webview: WebviewWindow, mut settings: Settings) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;
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
        shell::wallpaper()
    }

    pub fn set_native_shell_wallpaper(webview: WebviewWindow, path: PathBuf) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::ModifyConfiguration)?;
        shell::set_wallpaper(&path)
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
        register_webviews_bridge();
        PERFORMANCE_MODE.load()
    }
}
