#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::WlanBssEntry;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

use super::domain::{WifiEvent, WifiSystem};

#[cfg(windows)]
type Platform = windows::WindowsWifi;

/// Stateless: the adapters are tied to their OS watchers, so the native layer keeps them.
pub struct WifiManager {
    platform: Platform,
}

event_manager!(WifiManager, WifiEvent);

impl WifiManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<WifiManager> = LazyLock::new(|| {
            let mut m = WifiManager::create();
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
        self.platform.init(Self::send)
    }

    pub fn scan(&self) {
        self.platform.scan();
    }

    pub fn networks(&self) -> Result<Vec<WlanBssEntry>> {
        self.platform.networks()
    }

    /// Returns whether it connected. The networks list changes when it does.
    pub fn connect(&self, ssid: &str, password: Option<&str>, hidden: bool) -> Result<bool> {
        let connected = self.platform.connect(ssid, password, hidden)?;
        if connected {
            Self::send(WifiEvent::NetworksChanged);
        }
        Ok(connected)
    }

    pub fn forget(&self, ssid: &str) -> Result<()> {
        self.platform.forget(ssid)?;
        Self::send(WifiEvent::NetworksChanged);
        Ok(())
    }

    pub fn disconnect(&self) -> Result<()> {
        self.platform.disconnect()?;
        Self::send(WifiEvent::NetworksChanged);
        Ok(())
    }
}
