mod application;
mod domain;

use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Hotspot, NetworkAdapter},
};

use crate::{app::emit_to_webviews, error::Result};

use self::{application::NetworkManager, domain::NetworkEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static NetworkManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        NetworkManager::subscribe(|event| match event {
            NetworkEvent::AdaptersChanged { adapters } => {
                emit_to_webviews(SeelenEvent::NetworkAdapters, adapters);
            }
            NetworkEvent::InternetChanged { has_internet } => {
                emit_to_webviews(SeelenEvent::NetworkInternetConnection, has_internet);
            }
            NetworkEvent::HotspotChanged { hotspot } => {
                emit_to_webviews(SeelenEvent::NetworkHotspotChanged, hotspot);
            }
        });
    });
    NetworkManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_network_adapters() -> Result<Vec<NetworkAdapter>> {
        get_manager().adapters()
    }

    pub fn get_network_hotspot() -> Result<Option<Hotspot>> {
        get_manager().hotspot()
    }

    pub fn set_network_hotspot_state(enabled: bool) -> Result<()> {
        get_manager().set_hotspot_enabled(enabled)
    }

    pub fn get_network_internet_connection() -> Result<bool> {
        get_manager().has_internet()
    }
}
