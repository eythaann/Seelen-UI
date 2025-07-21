use std::path::PathBuf;

use itertools::Itertools;
use seelen_core::state::{
    IconPack, IconPackEntry, LauncherHistory, MonitorConfiguration, Plugin, Profile, Wallpaper,
    WallpaperInstanceSettings, WegItems, WegPinnedItemsVisibility, Widget,
};
use tauri_plugin_dialog::DialogExt;

use crate::{
    error_handler::Result,
    log_error,
    seelen::get_app_handle,
    trace_lock,
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
    windows_api::{window::Window, WindowsApi},
};

use super::{
    application::{FullState, FULL_STATE},
    domain::{AppConfig, Placeholder, Settings, Theme},
};

#[tauri::command(async)]
pub fn state_get_icon_packs() -> Vec<IconPack> {
    let mutex = FULL_STATE.load().icon_packs().clone();
    let icon_packs = trace_lock!(mutex);
    icon_packs.owned_list()
}

#[tauri::command(async)]
pub fn state_get_themes() -> Vec<Theme> {
    FULL_STATE.load().themes().values().cloned().collect_vec()
}

#[tauri::command(async)]
pub fn state_get_toolbar_items() -> Placeholder {
    FULL_STATE.load().toolbar_items().clone()
}

#[tauri::command(async)]
pub fn state_get_weg_items() -> WegItems {
    FULL_STATE.load().weg_items().clone()
}

#[tauri::command(async)]
pub fn state_write_weg_items(window: tauri::Window, mut items: WegItems) -> Result<()> {
    items.sanitize();
    let guard = FULL_STATE.load();

    let monitor = Window::from(window.hwnd()?.0 as isize).monitor();
    let device_id = monitor.stable_id()?;
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
    FULL_STATE.load().launcher_history().clone()
}

#[tauri::command(async)]
pub fn state_get_settings(path: Option<PathBuf>) -> Result<Settings> {
    if let Some(path) = path {
        let mut settings = FullState::get_settings_from_path(&path)?;
        settings.migrate()?;
        settings.sanitize()?;
        Ok(settings)
    } else {
        Ok(FULL_STATE.load().settings().clone())
    }
}

#[tauri::command(async)]
pub fn state_get_default_settings() -> Settings {
    Settings::default()
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
        .settings_by_app()
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
pub fn state_get_plugins() -> Vec<Plugin> {
    FULL_STATE.load().plugins().values().cloned().collect_vec()
}

#[tauri::command(async)]
pub fn state_get_widgets() -> Vec<Widget> {
    FULL_STATE.load().widgets().values().cloned().collect_vec()
}

#[tauri::command(async)]
pub fn state_get_profiles() -> Vec<Profile> {
    FULL_STATE.load().profiles.clone()
}

#[tauri::command(async)]
pub fn state_get_wallpapers() -> Vec<Wallpaper> {
    FULL_STATE.load().wallpapers.clone()
}

#[tauri::command(async)]
pub fn state_delete_cached_icons() -> Result<()> {
    let mutex = FULL_STATE.load().icon_packs().clone();
    let mut icon_manager = trace_lock!(mutex);
    icon_manager.clear_system_icons()?;
    icon_manager.sanitize_system_icon_pack(false)?;
    icon_manager.write_system_icon_pack()?;
    Ok(())
}

#[tauri::command(async)]
pub fn state_add_icon_to_custom_icon_pack(_icon: IconPackEntry) -> Result<()> {
    todo!()
}
