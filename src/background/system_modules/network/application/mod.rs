#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::{Hotspot, NetworkAdapter};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

use super::domain::{NetworkEvent, NetworkSysEvent, NetworkSystem};

#[cfg(windows)]
type Platform = windows::WindowsNetwork;

/// Stateless: everything is queried on demand.
pub struct NetworkManager {
    platform: Platform,
}

event_manager!(NetworkManager, NetworkEvent);

impl NetworkManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<NetworkManager> = LazyLock::new(|| {
            let mut m = NetworkManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.platform.init(Self::on_platform_event)
    }

    fn on_platform_event(event: NetworkSysEvent) {
        let manager = Self::instance();
        match event {
            NetworkSysEvent::AdaptersChanged => manager.send_adapters(),
            NetworkSysEvent::StatusChanged => {
                if let Some(has_internet) = manager.platform.has_internet().ok_logged() {
                    log::trace!(target: "network", "Internet connection: {has_internet}");
                    Self::send(NetworkEvent::InternetChanged { has_internet });
                }
                // the default route (`is_default`) may have moved to another adapter
                manager.send_adapters();
                if let Some(hotspot) = manager.platform.hotspot().ok_logged() {
                    Self::send(NetworkEvent::HotspotChanged { hotspot });
                }
            }
        }
    }

    fn send_adapters(&self) {
        if let Some(adapters) = self.platform.adapters().ok_logged() {
            Self::send(NetworkEvent::AdaptersChanged { adapters });
        }
    }

    pub fn adapters(&self) -> Result<Vec<NetworkAdapter>> {
        self.platform.adapters()
    }

    pub fn has_internet(&self) -> Result<bool> {
        self.platform.has_internet()
    }

    pub fn hotspot(&self) -> Result<Option<Hotspot>> {
        self.platform.hotspot()
    }

    pub fn set_hotspot_enabled(&self, enabled: bool) -> Result<()> {
        self.platform.set_hotspot_enabled(enabled)
    }
}
