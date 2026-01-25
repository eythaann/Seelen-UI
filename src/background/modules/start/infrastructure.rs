use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::StartMenuItem};

use crate::{app::emit_to_webviews, error::Result};

use super::application::StartMenuManager;

fn get_start_menu_manager() -> &'static StartMenuManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        StartMenuManager::subscribe(|_event| {
            if let Ok(items) = get_start_menu_items() {
                emit_to_webviews(SeelenEvent::StartMenuItemsChanged, items);
            }
        });
    });
    StartMenuManager::instance()
}

#[tauri::command(async)]
pub fn get_start_menu_items() -> Result<Vec<StartMenuItem>> {
    let manager = get_start_menu_manager();
    Ok(manager
        .list
        .to_vec()
        .into_iter()
        .map(|item| (*item).clone())
        .collect())
}
