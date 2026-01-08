use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::SystemLanguage};

use crate::{app::emit_to_webviews, error::Result};

use super::application::LanguageManager;

/// Lazy initialization wrapper that registers Tauri events on first access
/// This keeps Tauri logic separate from system logic while ensuring lazy initialization
fn get_language_manager() -> &'static LanguageManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        LanguageManager::subscribe(|_event| {
            emit_to_webviews(
                SeelenEvent::SystemLanguagesChanged,
                &LanguageManager::instance().get_languages(),
            );
        });
    });
    LanguageManager::instance()
}

#[tauri::command(async)]
pub fn get_system_languages() -> Result<Vec<SystemLanguage>> {
    get_language_manager();
    LanguageManager::enum_langs()
}

#[tauri::command(async)]
pub fn set_system_keyboard_layout(id: String, handle: String) -> Result<()> {
    get_language_manager();
    LanguageManager::set_keyboard_layout(&id, &handle)?;
    Ok(())
}
