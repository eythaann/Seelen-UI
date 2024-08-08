use itertools::Itertools;

use super::{
    application::FULL_STATE,
    domain::{AppConfig, Placeholder, Theme, WegItems},
};

#[tauri::command]
pub fn state_get_themes() -> Vec<Theme> {
    FULL_STATE.lock().themes().values().cloned().collect_vec()
}

#[tauri::command]
pub fn state_get_placeholders() -> Vec<Placeholder> {
    FULL_STATE
        .lock()
        .placeholders()
        .values()
        .cloned()
        .collect_vec()
}

#[tauri::command]
pub fn state_get_weg_items() -> WegItems {
    FULL_STATE.lock().weg_items().clone()
}

#[tauri::command]
pub fn state_get_specific_apps_configurations() -> Vec<AppConfig> {
    FULL_STATE
        .lock()
        .settings_by_app()
        .iter()
        .cloned()
        .collect_vec()
}
