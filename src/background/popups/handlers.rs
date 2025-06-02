use seelen_core::state::SluPopupConfig;
use uuid::Uuid;

use super::POPUPS_MANAGER;

#[tauri::command(async)]
pub fn get_popup_config(instance_id: Uuid) -> SluPopupConfig {
    POPUPS_MANAGER
        .lock()
        .configs
        .get(&instance_id)
        .cloned()
        .unwrap_or_default()
}
