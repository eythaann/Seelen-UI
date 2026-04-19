use std::sync::Once;

use seelen_core::handlers::SeelenEvent;

use crate::{app::emit_to_webviews, error::Result};

use super::application::FocusAssistManager;

fn get_focus_assist_manager() -> &'static FocusAssistManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        FocusAssistManager::subscribe(|mode| {
            emit_to_webviews(SeelenEvent::FocusAssistChanged, mode);
        });
    });
    FocusAssistManager::instance()
}

#[tauri::command(async)]
pub fn get_focus_assist() -> bool {
    get_focus_assist_manager().is_active()
}

#[tauri::command(async)]
pub fn set_focus_assist(enabled: bool) -> Result<()> {
    get_focus_assist_manager().set_focus_assist(enabled)
}
