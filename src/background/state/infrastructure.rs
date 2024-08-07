use itertools::Itertools;

use super::{
    application::FULL_STATE,
    domain::{Placeholder, Theme, WegItems},
    placeholders::PLACEHOLDERS_MANAGER,
    themes::THEME_MANAGER,
};

#[tauri::command]
pub fn state_get_themes() -> Vec<Theme> {
    THEME_MANAGER
        .lock()
        .themes()
        .values()
        .cloned()
        .collect_vec()
}

#[tauri::command]
pub fn state_get_placeholders() -> Vec<Placeholder> {
    PLACEHOLDERS_MANAGER
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
