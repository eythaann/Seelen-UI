use std::sync::{Arc, Once};

use seelen_core::{handlers::SeelenEvent, system_state::StartMenuItem};

use crate::app::emit_to_webviews;

use super::application::StartMenuManager;

fn get_start_menu_manager() -> &'static StartMenuManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        StartMenuManager::subscribe(|_event| {
            emit_to_webviews(SeelenEvent::StartMenuItemsChanged, get_start_menu_items());
        });
    });
    StartMenuManager::instance()
}

#[tauri::command(async)]
pub fn get_start_menu_items() -> Vec<Arc<StartMenuItem>> {
    let manager = get_start_menu_manager();
    manager.list.to_vec()
}
