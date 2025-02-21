use crate::error_handler::Result;

use super::{application::LanguageManager, domain::Language};

#[tauri::command(async)]
pub fn get_system_languages() -> Result<Vec<Language>> {
    LanguageManager::enum_langs()
}

#[tauri::command(async)]
pub fn set_system_keyboard_layout(id: String) -> Result<()> {
    LanguageManager::set_keyboard_layout(&id)
}
