use std::path::PathBuf;

use itertools::Itertools;
use seelen_core::state::{Plugin, Profile, WegItems, Widget, WindowManagerLayout};

use crate::{error_handler::Result, trace_lock, windows_api::WindowsApi};

use super::{
    application::{FullState, LauncherHistory, FULL_STATE},
    domain::{AppConfig, Placeholder, Settings, Theme},
};

#[tauri::command(async)]
pub fn state_get_themes() -> Vec<Theme> {
    FULL_STATE.load().themes().values().cloned().collect_vec()
}

#[tauri::command(async)]
pub fn state_get_placeholders() -> Vec<Placeholder> {
    FULL_STATE
        .load()
        .placeholders()
        .values()
        .cloned()
        .collect_vec()
}

#[tauri::command(async)]
pub fn state_get_layouts() -> Vec<WindowManagerLayout> {
    FULL_STATE.load().layouts().values().cloned().collect_vec()
}

#[tauri::command(async)]
pub fn state_get_weg_items() -> WegItems {
    let state = FULL_STATE.load();
    let items = trace_lock!(state.weg_items);
    items.clone()
}

#[tauri::command(async)]
pub fn state_get_history() -> LauncherHistory {
    FULL_STATE.load().launcher_history().clone()
}

#[tauri::command(async)]
pub fn state_get_settings(path: Option<PathBuf>) -> Result<Settings> {
    if let Some(path) = path {
        let mut settings = FullState::get_settings_from_path(&path)?;
        settings.sanitize();
        Ok(settings)
    } else {
        Ok(FULL_STATE.load().settings().clone())
    }
}

#[tauri::command(async)]
pub fn state_write_settings(settings: Settings) -> Result<()> {
    FULL_STATE.rcu(move |state| {
        let mut state = state.cloned();
        state.settings = settings.clone();
        state
    });
    FULL_STATE.load().write_settings()?;
    Ok(())
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
pub fn state_get_wallpaper() -> Result<PathBuf> {
    WindowsApi::get_wallpaper()
}

#[tauri::command(async)]
pub fn state_set_wallpaper(path: String) -> Result<()> {
    WindowsApi::set_wallpaper(path)
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
