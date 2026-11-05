mod application;
mod domain;

use std::sync::Once;

use seelen_core::handlers::SeelenEvent;

use crate::{app::emit_to_webviews, error::Result};

pub use self::application::WifiManager;
use self::domain::WifiEvent;

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static WifiManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        WifiManager::subscribe(|event| match event {
            WifiEvent::NetworksChanged => match WifiManager::instance().networks() {
                Ok(networks) => emit_to_webviews(SeelenEvent::NetworkWlanScanned, &networks),
                Err(e) => log::error!("Failed to emit WiFi networks: {e}"),
            },
        });
    });
    WifiManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn wlan_scan() {
        get_manager().scan();
    }

    pub fn wlan_connect(ssid: String, password: Option<String>, hidden: bool) -> Result<bool> {
        match get_manager().connect(&ssid, password.as_deref(), hidden) {
            Ok(connected) => Ok(connected),
            Err(err) => {
                log::error!("WiFi connect error: {err}");
                Ok(false)
            }
        }
    }

    pub fn wlan_forget(ssid: String) -> Result<()> {
        get_manager().forget(&ssid)
    }

    pub fn wlan_disconnect() -> Result<()> {
        get_manager().disconnect()
    }
}
