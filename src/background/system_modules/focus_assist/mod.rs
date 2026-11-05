mod application;
mod domain;

use std::sync::Once;

use seelen_core::handlers::SeelenEvent;

use crate::{app::emit_to_webviews, error::Result};

use self::{application::FocusAssistManager, domain::FocusAssistEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static FocusAssistManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        FocusAssistManager::subscribe(|event| match event {
            FocusAssistEvent::ActiveChanged { active } => {
                emit_to_webviews(SeelenEvent::FocusAssistChanged, active);
            }
        });
    });
    FocusAssistManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_focus_assist() -> bool {
        get_manager().is_active()
    }

    pub fn set_focus_assist(enabled: bool) -> Result<()> {
        get_manager().set_active(enabled)
    }
}
