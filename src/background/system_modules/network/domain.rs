use seelen_core::system_state::{Hotspot, NetworkAdapter};

use crate::error::Result;

/// OS contract of the network module.
pub trait NetworkSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening network changes, reported through `on_event`.
    /// Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(NetworkSysEvent) + Send + Sync + 'static;

    fn adapters(&self) -> Result<Vec<NetworkAdapter>>;

    /// Whether there is internet access, through IPv4 or IPv6.
    fn has_internet(&self) -> Result<bool>;

    /// `None` when hotspot/tethering is not supported or not available on the current
    /// connection (e.g. no active internet connection).
    fn hotspot(&self) -> Result<Option<Hotspot>>;

    fn set_hotspot_enabled(&self, enabled: bool) -> Result<()>;
}

/// Raw OS notifications: they only signal what changed, the manager queries the new data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkSysEvent {
    /// An interface was added, removed or changed its configuration.
    AdaptersChanged,
    /// The connectivity (internet access, connection profile, ...) changed.
    StatusChanged,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum NetworkEvent {
    AdaptersChanged { adapters: Vec<NetworkAdapter> },
    InternetChanged { has_internet: bool },
    HotspotChanged { hotspot: Option<Hotspot> },
}
