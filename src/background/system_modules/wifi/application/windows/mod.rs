mod adapter;
mod state;

use seelen_core::system_state::WlanBssEntry;

use crate::{
    error::Result,
    system_modules::wifi::domain::{WifiEvent, WifiSystem},
};

use self::state::{RawWifiEvent, WifiState};

/// Facade over [`WifiState`], the singleton owning the adapters watchers.
pub struct WindowsWifi {
    subscription: Option<String>,
}

impl WifiSystem for WindowsWifi {
    fn create() -> Self {
        Self { subscription: None }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(WifiEvent) + Send + Sync + 'static,
    {
        WifiState::instance();
        self.subscription = Some(WifiState::subscribe(move |event| match event {
            RawWifiEvent::NetworksChanged => on_event(WifiEvent::NetworksChanged),
        }));
        Ok(())
    }

    fn scan(&self) {
        WifiState::instance().scan_networks();
    }

    fn networks(&self) -> Result<Vec<WlanBssEntry>> {
        WifiState::instance().get_available_networks()
    }

    fn connect(&self, ssid: &str, password: Option<&str>, hidden: bool) -> Result<bool> {
        WifiState::instance().connect(ssid, password, hidden)
    }

    fn forget(&self, ssid: &str) -> Result<()> {
        WifiState::instance().forget(ssid)
    }

    fn disconnect(&self) -> Result<()> {
        WifiState::instance().disconnect()
    }
}

impl Drop for WindowsWifi {
    fn drop(&mut self) {
        if let Some(id) = self.subscription.take() {
            WifiState::unsubscribe(&id);
        }
    }
}
