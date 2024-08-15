use std::path::PathBuf;

use itertools::Itertools;

use crate::{error_handler::Result, trace_lock, windows_api::WindowsApi};

use super::{
    application::FULL_STATE,
    domain::{AppConfig, Placeholder, Settings, Theme, WegItems},
};

#[tauri::command]
pub fn state_get_themes() -> Vec<Theme> {
    trace_lock!(FULL_STATE)
        .themes()
        .values()
        .cloned()
        .collect_vec()
}

#[tauri::command]
pub fn state_get_placeholders() -> Vec<Placeholder> {
    trace_lock!(FULL_STATE)
        .placeholders()
        .values()
        .cloned()
        .collect_vec()
}

#[tauri::command]
pub fn state_get_weg_items() -> WegItems {
    trace_lock!(FULL_STATE).weg_items().clone()
}

#[tauri::command]
pub fn state_get_settings() -> Settings {
    trace_lock!(FULL_STATE).settings().clone()
}

#[tauri::command]
pub fn state_get_specific_apps_configurations() -> Vec<AppConfig> {
    trace_lock!(FULL_STATE)
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
