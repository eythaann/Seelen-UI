use seelen_core::system_state::WlanBssEntry;

use crate::error::Result;

/// OS contract of the WiFi module.
pub trait WifiSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts watching the adapters, reporting the scan results through `on_event`.
    /// Called once; the watching lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(WifiEvent) + Send + Sync + 'static;

    /// Triggers a scan on every adapter, results are reported as `NetworksChanged`.
    fn scan(&self);

    /// Visible access points, with their security and known/connected status.
    fn networks(&self) -> Result<Vec<WlanBssEntry>>;

    /// Returns whether it connected.
    fn connect(&self, ssid: &str, password: Option<&str>, hidden: bool) -> Result<bool>;

    /// Deletes the saved profile of the network.
    fn forget(&self, ssid: &str) -> Result<()>;

    fn disconnect(&self) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiEvent {
    NetworksChanged,
}
