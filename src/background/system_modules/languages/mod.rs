mod application;
mod domain;

use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{ImeState, SystemLanguage},
};

use crate::{app::emit_to_webviews, error::Result};

use self::{application::LanguageManager, domain::LanguageEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static LanguageManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        LanguageManager::subscribe(|event| match event {
            LanguageEvent::LayoutChanged => {
                emit_to_webviews(
                    SeelenEvent::SystemLanguagesChanged,
                    LanguageManager::instance().languages(),
                );
            }
            LanguageEvent::ImeChanged => {
                if let Ok(state) = LanguageManager::instance().ime_state() {
                    emit_to_webviews(SeelenEvent::SystemImeStateChanged, state);
                }
            }
        });
    });
    LanguageManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_system_languages() -> Vec<SystemLanguage> {
        get_manager().languages()
    }

    pub fn get_ime_state() -> Result<ImeState> {
        get_manager().ime_state()
    }

    pub fn set_system_keyboard_layout(id: String, handle: String) -> Result<()> {
        get_manager().set_keyboard_layout(&id, &handle)
    }
}
