use crate::error_handler::Result;

use super::{application::LanguageManager, domain::Language};

#[tauri::command(async)]
pub fn get_system_languages() -> Result<Vec<Language>> {
    LanguageManager::enum_langs()
}
