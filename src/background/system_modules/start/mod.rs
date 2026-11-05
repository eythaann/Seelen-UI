mod application;
mod domain;

use std::sync::{Arc, Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{StartMenuItem, StartMenuLayout},
};

use crate::{app::emit_to_webviews, error::Result};

pub use self::{application::StartMenuManager, domain::StartMenuEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static StartMenuManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        StartMenuManager::subscribe(|event| match event {
            StartMenuEvent::ItemsRefreshed => {
                emit_to_webviews(
                    SeelenEvent::StartMenuItemsChanged,
                    StartMenuManager::instance().get_all(),
                );
            }
        });
    });
    StartMenuManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_start_menu_items() -> Vec<Arc<StartMenuItem>> {
        get_manager().get_all()
    }

    pub async fn get_native_start_menu() -> Result<StartMenuLayout> {
        get_manager().native_layout().await
    }
}
