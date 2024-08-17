use std::path::PathBuf;

use itertools::Itertools;

use crate::{error_handler::Result, windows_api::WindowsApi};

use super::{
    application::FULL_STATE,
    domain::{AppConfig, Placeholder, Settings, Theme, WegItems},
};

#[tauri::command]
pub fn state_get_themes() -> Vec<Theme> {
    FULL_STATE.load().themes().values().cloned().collect_vec()
}

#[tauri::command]
pub fn state_get_placeholders() -> Vec<Placeholder> {
    FULL_STATE
        .load()
        .placeholders()
        .values()
        .cloned()
        .collect_vec()
}

#[tauri::command]
pub fn state_get_weg_items() -> WegItems {
    FULL_STATE.load().weg_items().clone()
}

#[tauri::command]
pub fn state_get_settings() -> Settings {
    FULL_STATE.load().settings().clone()
}

#[tauri::command]
pub fn state_get_specific_apps_configurations() -> Vec<AppConfig> {
    FULL_STATE
        .load()
        .settings_by_app()
        .iter()
        .cloned()
        .collect_vec()
}

#[tauri::command]
pub fn state_get_wallpaper() -> Result<PathBuf> {
    WindowsApi::get_wallpaper()
}

#[tauri::command]
pub fn state_set_wallpaper(path: String) -> Result<()> {
    WindowsApi::set_wallpaper(path)
}
