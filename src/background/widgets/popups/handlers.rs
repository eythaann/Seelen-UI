use seelen_core::state::SluPopupConfig;
use uuid::Uuid;

use crate::error::Result;

use super::POPUPS_MANAGER;

#[tauri::command(async)]
pub fn get_popup_config(instance_id: Uuid) -> Option<SluPopupConfig> {
    POPUPS_MANAGER.lock().configs.get(&instance_id).cloned()
}

#[tauri::command(async)]
pub fn create_popup(config: SluPopupConfig) -> Result<Uuid> {
    POPUPS_MANAGER.lock().create(config)
}

#[tauri::command(async)]
pub fn update_popup(instance_id: Uuid, config: SluPopupConfig) -> Result<()> {
    POPUPS_MANAGER.lock().update(&instance_id, config)?;
    Ok(())
}

#[tauri::command(async)]
pub fn close_popup(instance_id: Uuid) -> Result<()> {
    POPUPS_MANAGER.lock().close_popup(&instance_id)?;
    Ok(())
}
