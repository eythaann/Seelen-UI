use std::path::PathBuf;

use itertools::Itertools;
use seelen_core::state::{
    by_monitor::MonitorConfiguration, by_wallpaper::WallpaperInstanceSettings, IconPackEntry,
    LauncherHistory, PerformanceMode, Profile, Wallpaper, WegItems, WegPinnedItemsVisibility,
};
use tauri_plugin_dialog::DialogExt;

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    state::application::performance::PERFORMANCE_MODE,
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
    windows_api::{window::Window, WindowsApi},
};

use super::{
    application::FULL_STATE,
    domain::{AppConfig, Placeholder, Settings},
};

#[tauri::command(async)]
pub fn state_get_toolbar_items() -> Placeholder {
    FULL_STATE.load().toolbar_items.clone()
}

#[tauri::command(async)]
pub fn state_write_weg_items(window: tauri::Window, mut items: WegItems) -> Result<()> {
    items.sanitize();
    let guard = FULL_STATE.load();

    let monitor = Window::from(window.hwnd()?.0 as isize).monitor();
    let device_id = monitor.stable_id2()?;
    if guard.get_weg_pinned_item_visibility(&device_id) == WegPinnedItemsVisibility::WhenPrimary
        && !monitor.is_primary()
        || items == guard.weg_items
    {
        return Ok(());
    }
    guard.write_weg_items(&items)?;
    Ok(())
}

#[tauri::command(async)]
pub fn state_get_history() -> LauncherHistory {
    FULL_STATE.load().launcher_history.clone()
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
pub fn state_write_settings(settings: Settings) -> Result<()> {
    FULL_STATE.rcu(move |state| {
        let mut state = state.cloned();
        state.settings = settings.clone();
        state
    });
    FULL_STATE.load().write_settings()
}

#[tauri::command(async)]
pub fn state_get_specific_apps_configurations() -> Vec<AppConfig> {
    FULL_STATE
        .load()
        .settings_by_app
        .iter()
        .cloned()
        .collect_vec()
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
                    log_error!(Wallpaper::create_from_file(&path, &folder_to_store, true));
                }
            }
        });
    Ok(())
}

#[tauri::command(async)]
pub fn state_get_profiles() -> Vec<Profile> {
    FULL_STATE.load().profiles.clone()
}

#[tauri::command(async)]
pub fn state_add_icon_to_custom_icon_pack(_icon: IconPackEntry) -> Result<()> {
    todo!()
}

#[tauri::command(async)]
pub fn state_get_performance_mode() -> PerformanceMode {
    **PERFORMANCE_MODE.load()
}
