use seelen_core::system_state::{RadioDevice, RadioDeviceKind};

use crate::error::Result;

/// OS contract of the radios module (WiFi, Bluetooth, ... on/off switches).
pub trait RadiosSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts watching the radios, reporting changes through `on_event`.
    /// Called once; the watching lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(RadiosSysEvent) + Send + Sync + 'static;

    fn radios(&self) -> Vec<RadioDevice>;

    /// Turns on/off every radio of the kind.
    fn set_enabled(&self, kind: RadioDeviceKind, enabled: bool) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadiosSysEvent {
    /// A radio was added, removed or changed its state.
    Changed,
    /// A radio was turned on (also reported as `Changed`).
    Enabled { kind: RadioDeviceKind },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadiosEvent {
    Changed,
}
