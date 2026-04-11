use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::ClipboardData};

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    modules::clipboard::application::ClipboardManager,
    windows_api::input::Keyboard,
};

fn get_clipboard_manager() -> &'static ClipboardManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        ClipboardManager::subscribe(|_| {
            emit_to_webviews(
                SeelenEvent::ClipboardDataChanged,
                ClipboardManager::instance().get_data(),
            );
        });
    });
    ClipboardManager::instance()
}

#[tauri::command(async)]
pub fn clipboard_get_data() -> ClipboardData {
    get_clipboard_manager().get_data()
}

#[tauri::command(async)]
pub fn clipboard_delete_entry(id: String) -> Result<()> {
    get_clipboard_manager();
    ClipboardManager::delete_entry(&id)
}

#[tauri::command(async)]
pub fn clipboard_clear_history() -> Result<()> {
    get_clipboard_manager();
    ClipboardManager::clear_history()
}

#[tauri::command(async)]
pub fn clipboard_set_content(id: String) -> Result<()> {
    get_clipboard_manager();
    ClipboardManager::set_clipboard_content(&id)
}

/// Sets the given history entry as current clipboard content and simulates Ctrl+V
/// so the previously focused window receives a paste event.
/// A small delay is introduced to allow the widget to fully hide and focus to
/// return to the target window before the keystrokes are sent.
#[tauri::command(async)]
pub fn clipboard_paste(id: String) -> Result<()> {
    get_clipboard_manager();
    ClipboardManager::set_clipboard_content(&id)?;
    Keyboard::new().send_keys("{ctrl}v").log_error();
    Ok(())
}
