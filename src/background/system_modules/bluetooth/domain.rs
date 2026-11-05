use std::future::Future;

use seelen_core::system_state::{BluetoothDevice, DevicePairingAnswer, DevicePairingNeededAction};

use crate::error::Result;

/// OS contract of the bluetooth module (devices, discovery and pairing).
pub trait BluetoothSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts watching the devices, reporting changes through `on_event`.
    /// Called once; the watching lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(BluetoothEvent) + Send + Sync + 'static;

    /// Paired devices, plus the discovered ones while scanning.
    fn devices(&self) -> Vec<BluetoothDevice>;

    fn start_scanning(&self) -> Result<()>;

    fn stop_scanning(&self) -> Result<()>;

    /// The discovery doesn't reliably resume after the radio is turned back on.
    fn restart_scanning_if_active(&self) -> Result<()>;

    /// Starts pairing, returning what the user has to do to complete it.
    fn request_pair(
        &self,
        id: String,
    ) -> impl Future<Output = Result<DevicePairingNeededAction>> + Send;

    /// Completes (or rejects) a pairing started by [`Self::request_pair`].
    fn confirm_pairing(
        &self,
        id: String,
        answer: DevicePairingAnswer,
    ) -> impl Future<Output = Result<()>> + Send;

    fn connect(&self, id: &str) -> Result<()>;

    fn disconnect(&self, id: &str) -> Result<()>;

    /// Unpairs the device.
    fn forget(&self, id: String) -> impl Future<Output = Result<()>> + Send;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothEvent {
    DevicesChanged,
}
