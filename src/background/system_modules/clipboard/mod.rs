mod application;
mod domain;

use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::ClipboardData};

use crate::{app::emit_to_webviews, error::Result};

use self::{application::ClipboardManager, domain::ClipboardEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static ClipboardManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        ClipboardManager::subscribe(|event| match event {
            ClipboardEvent::EntriesChanged | ClipboardEvent::EnabledChanged => {
                emit_to_webviews(
                    SeelenEvent::ClipboardDataChanged,
                    ClipboardManager::instance().data(),
                );
            }
        });
    });
    ClipboardManager::instance()
}

/// Starts the clipboard manager (history restore + capture) without registering
/// the webview bridge, which still happens lazily on first command access.
/// Called at startup so history is captured even if no widget requests it.
pub fn init_clipboard_manager() {
    ClipboardManager::instance();
}

impl crate::tauri_handlers::Handlers {
    pub fn clipboard_get_data() -> ClipboardData {
        get_manager().data()
    }

    pub fn clipboard_delete_entry(id: String) -> Result<()> {
        get_manager().delete_entry(&id);
        Ok(())
    }

    pub fn clipboard_clear_history() -> Result<()> {
        get_manager().clear_history();
        Ok(())
    }

    pub fn clipboard_pin_entry(id: String) -> Result<()> {
        get_manager().pin_entry(&id)
    }

    pub fn clipboard_unpin_entry(id: String) -> Result<()> {
        get_manager().unpin_entry(&id);
        Ok(())
    }

    pub fn clipboard_set_content(id: String) -> Result<()> {
        get_manager().set_clipboard_content(&id, false)
    }

    /// Sets the given history entry as current clipboard content and simulates the paste
    /// shortcut so the previously focused window receives a paste event.
    ///
    /// When `plain` is `true`, only plain text is placed on the clipboard, so the
    /// target app receives plain text instead of any rich (HTML/RTF/...) content.
    pub fn clipboard_paste(id: String, plain: bool) -> Result<()> {
        get_manager().paste(&id, plain)
    }
}
