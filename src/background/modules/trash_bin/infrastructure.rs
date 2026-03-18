use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::TrashBinInfo};

use crate::{app::emit_to_webviews, error::Result, utils::lock_free::TracedMutex};

use super::application::{TrashBinManager, TrashBinManagerEvent};

fn get_trash_bin_manager() -> &'static TracedMutex<TrashBinManager> {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        TrashBinManager::subscribe(|event| {
            let TrashBinManagerEvent::InfoChanged(info) = event;
            emit_to_webviews(SeelenEvent::TrashBinChanged, info);
        });
    });
    TrashBinManager::instance()
}

#[tauri::command(async)]
pub fn get_trash_bin_info() -> TrashBinInfo {
    get_trash_bin_manager().lock().info.clone()
}

#[tauri::command(async)]
pub fn trash_bin_empty() -> Result<()> {
    get_trash_bin_manager();
    TrashBinManager::empty()
}
