use std::thread;

use itertools::Itertools;
use seelen_core::{
    handlers::SeelenEvent,
    system_state::{BluetoothDevice, BluetoothDevicePairShowPinRequest},
};
use tauri::Emitter;

use crate::{
    app::get_app_handle,
    log_error,
    modules::bluetooth::{
        bluetooth_pair_manager::{BluetoothPairEvent, BluetoothPairManager},
        BluetoothEvent, BluetoothManager, BLUETOOTH_MANAGER,
    },
    trace_lock,
};

use crate::error::Result;

pub fn register_bluetooth_events() {
    /* std::thread::spawn(|| {
        log_error!(trace_lock!(BLUETOOTH_MANAGER).register_for_bt_devices());

        BluetoothManager::subscribe(|event| match event {
            BluetoothEvent::DevicesChanged(items) => {
                log_error!(get_app_handle().emit(
                    SeelenEvent::BluetoothDevicesChanged,
                    items
                        .into_iter()
                        .map_into()
                        .collect::<Vec<BluetoothDevice>>()
                ));
            }
            BluetoothEvent::DiscoveredDevicesChanged(items) => {
                log_error!(get_app_handle().emit(
                    SeelenEvent::BluetoothDiscoveredDevicesChanged,
                    items
                        .into_iter()
                        .map_into()
                        .collect::<Vec<BluetoothDevice>>()
                ));
            }
        });
        BluetoothPairManager::subscribe(|event| match event {
            BluetoothPairEvent::ShowPin(pin, confirmation_needed) => {
                thread::spawn(move || {
                    log_error!(get_app_handle().emit(
                        SeelenEvent::BluetoothPairShowPin,
                        BluetoothDevicePairShowPinRequest {
                            pin,
                            confirmation_needed
                        }
                    ));
                });
            }
            BluetoothPairEvent::RequestPin() => {
                log_error!(get_app_handle().emit(SeelenEvent::BluetoothPairRequestPin, ()));
            }
            BluetoothPairEvent::Confirm(_, _) => {
                // Do not need anything, this is an internal event that is triggered by UI confirmation
            }
        });
    }); */
}

#[tauri::command(async)]
pub fn get_connected_bluetooth_devices() -> Result<Vec<BluetoothDevice>> {
    let manager = trace_lock!(BLUETOOTH_MANAGER);
    let collection = manager
        .known_items
        .values()
        .cloned()
        .map_into()
        .collect::<Vec<BluetoothDevice>>();
    Ok(collection)
}

#[tauri::command(async)]
pub fn start_bluetooth_scanning() -> Result<()> {
    let mut manager = trace_lock!(BLUETOOTH_MANAGER);
    manager.discover()?;
    Ok(())
}

#[tauri::command(async)]
pub fn stop_bluetooth_scanning() -> Result<()> {
    let mut manager = trace_lock!(BLUETOOTH_MANAGER);
    manager.stop_discovery()?;
    Ok(())
}

#[tauri::command(async)]
pub async fn pair_bluetooth_device(address: u64) -> Result<()> {
    BluetoothPairManager::pair(address).await
}

#[tauri::command(async)]
pub async fn forget_bluetooth_device(id: String) -> Result<()> {
    BluetoothPairManager::forget(id).await
}

#[tauri::command(async)]
pub fn confirm_bluetooth_device_pair(accept: bool, passphrase: String) -> Result<()> {
    //TODO(Eythaan): this part was never tested.
    log_error!(
        BluetoothPairManager::event_tx().send(BluetoothPairEvent::Confirm(accept, passphrase))
    );
    Ok(())
}
