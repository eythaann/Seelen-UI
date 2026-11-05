mod application;
mod domain;

use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::TrashBinInfo};

use crate::{app::emit_to_webviews, error::Result};

use self::{application::TrashBinManager, domain::TrashBinEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static TrashBinManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        TrashBinManager::subscribe(|event| match event {
            TrashBinEvent::InfoChanged { info } => {
                emit_to_webviews(SeelenEvent::TrashBinChanged, info);
            }
        });
    });
    TrashBinManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_trash_bin_info() -> TrashBinInfo {
        get_manager().info()
    }

    pub fn trash_bin_empty() -> Result<()> {
        get_manager().empty()
    }
}
