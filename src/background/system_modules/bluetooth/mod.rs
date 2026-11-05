mod application;
mod domain;

use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{BluetoothDevice, DevicePairingAnswer, DevicePairingNeededAction},
};

use crate::{app::emit_to_webviews, error::Result};

pub use self::application::BluetoothManager;
use self::domain::BluetoothEvent;

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static BluetoothManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        BluetoothManager::subscribe(|event| match event {
            BluetoothEvent::DevicesChanged => {
                emit_to_webviews(
                    SeelenEvent::BluetoothDevicesChanged,
                    BluetoothManager::instance().devices(),
                );
            }
        });
    });
    BluetoothManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_bluetooth_devices() -> Vec<BluetoothDevice> {
        get_manager().devices()
    }

    pub fn start_bluetooth_scanning() -> Result<()> {
        get_manager().start_scanning()
    }

    pub fn stop_bluetooth_scanning() -> Result<()> {
        get_manager().stop_scanning()
    }

    pub async fn request_pair_bluetooth_device(id: String) -> Result<DevicePairingNeededAction> {
        get_manager().request_pair(id).await
    }

    pub async fn confirm_bluetooth_device_pairing(
        id: String,
        answer: DevicePairingAnswer,
    ) -> Result<()> {
        get_manager().confirm_pairing(id, answer).await
    }

    pub fn disconnect_bluetooth_device(id: String) -> Result<()> {
        get_manager().disconnect(&id)
    }

    pub fn connect_bluetooth_device(id: String) -> Result<()> {
        get_manager().connect(&id)
    }

    pub async fn forget_bluetooth_device(id: String) -> Result<()> {
        get_manager().forget(id).await
    }
}
