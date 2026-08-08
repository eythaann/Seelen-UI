use std::{collections::HashSet, path::PathBuf};

use seelen_core::state::{
    by_monitor::MonitorConfiguration, by_wallpaper::WallpaperInstanceSettings, AppConfig,
    IconPackEntry, PerformanceMode, Settings, ToolbarState, Wallpaper, WegItems,
};
use tauri_plugin_dialog::DialogExt;

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    resources::RESOURCES,
    state::application::{performance::PERFORMANCE_MODE, BUNDLED_SETTINGS_BY_APP},
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
    windows_api::WindowsApi,
};

use super::application::{FULL_STATE, TOOLBAR_ITEMS_MANAGER, WEG_ITEMS_MANAGER};

#[tauri::command(async)]
pub fn state_get_toolbar_items() -> ToolbarState {
    TOOLBAR_ITEMS_MANAGER.get()
}

#[tauri::command(async)]
pub fn state_write_toolbar_items(items: ToolbarState) -> Result<()> {
    TOOLBAR_ITEMS_MANAGER.write(items)
}

#[tauri::command(async)]
pub fn state_get_weg_items() -> WegItems {
    WEG_ITEMS_MANAGER.get()
}

#[tauri::command(async)]
pub fn state_write_weg_items(items: WegItems) -> Result<()> {
    WEG_ITEMS_MANAGER.write(items)
}

#[tauri::command(async)]
pub fn state_get_settings(path: Option<PathBuf>) -> Result<Settings> {
    if let Some(path) = path {
        Ok(Settings::load(path)?)
    } else {
        Ok(FULL_STATE.load().settings.clone())
    }
}

#[tauri::command(async)]
pub fn state_get_default_settings() -> Result<Settings> {
    let mut settings = Settings::default();
    settings.sanitize()?;
    Ok(settings)
}

#[tauri::command(async)]
pub fn state_get_default_monitor_settings() -> MonitorConfiguration {
    MonitorConfiguration::default()
}

#[tauri::command(async)]
pub fn state_get_default_wallpaper_settings() -> WallpaperInstanceSettings {
    WallpaperInstanceSettings::default()
}

#[tauri::command(async)]
pub fn state_write_settings(mut settings: Settings) -> Result<()> {
    settings.sanitize()?;
    let previous = FULL_STATE.load().settings.clone();
    let reconcile_widgets = widget_topology_changed(&previous, &settings);
    FULL_STATE.rcu(move |state| {
        let mut state = state.cloned();
        state.settings = settings.clone();
        state
    });
    FULL_STATE
        .load()
        .write_settings_with_reconcile(reconcile_widgets)?;
    crate::backups::application::on_settings_saved();
    Ok(())
}

fn widget_topology_changed(previous: &Settings, next: &Settings) -> bool {
    let monitor_ids: HashSet<_> = previous
        .monitors_v3
        .keys()
        .chain(next.monitors_v3.keys())
        .cloned()
        .collect();

    RESOURCES
        .widgets
        .any_sync(|widget_id, _| {
            if previous.is_widget_enabled(widget_id) != next.is_widget_enabled(widget_id) {
                return true;
            }

            let previous_instances = previous
                .by_widget
                .others
                .get(widget_id)
                .and_then(|config| config.instances.as_ref());
            let next_instances = next
                .by_widget
                .others
                .get(widget_id)
                .and_then(|config| config.instances.as_ref());
            if previous_instances.map(|instances| instances.keys().cloned().collect::<HashSet<_>>())
                != next_instances.map(|instances| instances.keys().cloned().collect::<HashSet<_>>())
            {
                return true;
            }

            monitor_ids.iter().any(|monitor_id| {
                previous.is_widget_enabled_on_monitor(widget_id, monitor_id)
                    != next.is_widget_enabled_on_monitor(widget_id, monitor_id)
            })
        })
        .is_some()
}

#[tauri::command(async)]
pub fn state_get_settings_by_app() -> Vec<AppConfig> {
    BUNDLED_SETTINGS_BY_APP.iter().cloned().collect()
}

#[tauri::command(async)]
pub fn get_native_shell_wallpaper() -> Result<PathBuf> {
    WindowsApi::get_wallpaper()
}

#[tauri::command(async)]
pub fn set_native_shell_wallpaper(path: String) -> Result<()> {
    WindowsApi::set_wallpaper(path)
}

#[tauri::command(async)]
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

#[tauri::command(async)]
pub fn state_add_icon_to_custom_icon_pack(_icon: IconPackEntry) -> Result<()> {
    todo!()
}

#[tauri::command(async)]
pub fn state_get_performance_mode() -> PerformanceMode {
    PERFORMANCE_MODE.load()
}
