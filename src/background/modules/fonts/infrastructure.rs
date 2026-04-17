use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::SeelenFont};

use crate::{app::emit_to_webviews, error::Result};

use super::application::{FontManager, FontManagerEvent};

fn get_font_manager() -> &'static FontManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        FontManager::subscribe(|event| {
            if event == FontManagerEvent::FontsChanged {
                if let Ok(fonts) = FontManager::instance().get_fonts() {
                    emit_to_webviews(SeelenEvent::SystemFontsChanged, &fonts);
                }
            }
        });
    });
    FontManager::instance()
}

#[tauri::command(async)]
pub fn get_fonts() -> Result<Vec<SeelenFont>> {
    get_font_manager().get_fonts()
}
