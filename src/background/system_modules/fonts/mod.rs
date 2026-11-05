mod application;
mod domain;

use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::SeelenFont};

use crate::{app::emit_to_webviews, error::Result};

use self::{application::FontsManager, domain::FontsEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static FontsManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        FontsManager::subscribe(|event| match event {
            FontsEvent::Changed { fonts } => {
                emit_to_webviews(SeelenEvent::SystemFontsChanged, fonts);
            }
        });
    });
    FontsManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_fonts() -> Result<Vec<SeelenFont>> {
        get_manager().fonts()
    }
}
