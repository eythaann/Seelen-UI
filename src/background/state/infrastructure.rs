use itertools::Itertools;

use super::{domain::Theme, themes::THEME_MANAGER};

#[tauri::command]
pub fn state_get_themes() -> Vec<Theme> {
    THEME_MANAGER
        .lock()
        .themes()
        .values()
        .cloned()
        .collect_vec()
}
