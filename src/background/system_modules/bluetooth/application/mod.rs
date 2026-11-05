#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::{BluetoothDevice, DevicePairingAnswer, DevicePairingNeededAction};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

use super::domain::{BluetoothEvent, BluetoothSystem};

#[cfg(windows)]
type Platform = windows::WindowsBluetooth;

/// Stateless: the devices are tied to their OS watchers, so the native layer keeps them.
pub struct BluetoothManager {
    platform: Platform,
}

event_manager!(BluetoothManager, BluetoothEvent);

impl BluetoothManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<BluetoothManager> = LazyLock::new(|| {
            let mut m = BluetoothManager::create();
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

    pub fn devices(&self) -> Vec<BluetoothDevice> {
        self.platform.devices()
    }

    pub fn start_scanning(&self) -> Result<()> {
        self.platform.start_scanning()
    }

    pub fn stop_scanning(&self) -> Result<()> {
        self.platform.stop_scanning()
    }

    pub fn restart_scanning_if_active(&self) -> Result<()> {
        self.platform.restart_scanning_if_active()
    }

    pub async fn request_pair(&self, id: String) -> Result<DevicePairingNeededAction> {
        self.platform.request_pair(id).await
    }

    pub async fn confirm_pairing(&self, id: String, answer: DevicePairingAnswer) -> Result<()> {
        self.platform.confirm_pairing(id, answer).await
    }

    pub fn connect(&self, id: &str) -> Result<()> {
        self.platform.connect(id)
    }

    pub fn disconnect(&self, id: &str) -> Result<()> {
        self.platform.disconnect(id)
    }

    pub async fn forget(&self, id: String) -> Result<()> {
        self.platform.forget(id).await
    }
}
