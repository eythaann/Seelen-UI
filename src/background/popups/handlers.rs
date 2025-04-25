use seelen_core::state::SluPopupConfig;
use uuid::Uuid;

use super::POPUPS_MANAGER;

#[tauri::command(async)]
pub fn get_popup_config(id: Uuid) -> SluPopupConfig {
    POPUPS_MANAGER
        .lock()
        .configs
        .get(&id)
        .cloned()
        .unwrap_or_default()
}
