use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{error_handler::Result, seelen::get_app_handle, trace_lock};

use super::{
    application::{LanguageEvent, LanguageManager, LANGUAGE_MANAGER},
    domain::Language,
};

pub fn register_language_events() {
    std::thread::spawn(move || {
        trace_lock!(LANGUAGE_MANAGER)
            .init()
            .expect("Failed to initialize power manager");
    });

    LanguageManager::subscribe(|event| match event {
        LanguageEvent::KeyboardLayoutChanged(hkl) => {
            let mut lang_manager = trace_lock!(LANGUAGE_MANAGER);
            if !lang_manager.update_active(hkl) {
                lang_manager.languages = match LanguageManager::enum_langs() {
                    Ok(languages) => languages,
                    Err(e) => {
                        log::error!("Failed to enumerate languages: {}", e);
                        return;
                    }
                };
            }
            get_app_handle()
                .emit(SeelenEvent::SystemLanguagesChanged, &lang_manager.languages)
                .unwrap();
        }
    });
}

#[tauri::command(async)]
pub fn get_system_languages() -> Result<Vec<Language>> {
    LanguageManager::enum_langs()
}

#[tauri::command(async)]
pub fn set_system_keyboard_layout(id: String, handle: String) -> Result<()> {
    LanguageManager::set_keyboard_layout(&id, &handle)?;
    Ok(())
}
