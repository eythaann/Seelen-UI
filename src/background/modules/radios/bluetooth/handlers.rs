use seelen_core::{
    handlers::SeelenEvent,
    system_state::{BluetoothDevice, DevicePairingAnswer, DevicePairingNeededAction},
};
use tauri::Emitter;
use windows::Devices::Enumeration::{DevicePairingResultStatus, DeviceUnpairingResultStatus};

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    modules::radios::bluetooth::manager::BluetoothManager,
};

fn get_bluetooth_manager() -> &'static BluetoothManager {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        BluetoothManager::subscribe(|_e| {
            get_app_handle()
                .emit(
                    SeelenEvent::BluetoothDevicesChanged,
                    BluetoothManager::instance().get_all_devices(),
                )
                .log_error();
        });
    });
    BluetoothManager::instance()
}

#[tauri::command(async)]
pub fn get_bluetooth_devices() -> Vec<BluetoothDevice> {
    get_bluetooth_manager().get_all_devices()
}

#[tauri::command(async)]
pub fn start_bluetooth_scanning() -> Result<()> {
    get_bluetooth_manager().start_scanning()
}

#[tauri::command(async)]
pub fn stop_bluetooth_scanning() -> Result<()> {
    get_bluetooth_manager().stop_scanning()
}

#[tauri::command(async)]
pub async fn request_pair_bluetooth_device(id: String) -> Result<DevicePairingNeededAction> {
    let manager = get_bluetooth_manager();
    manager.request_pair_device(&id).await
}

#[tauri::command(async)]
pub async fn confirm_bluetooth_device_pairing(
    id: String,
    answer: DevicePairingAnswer,
) -> Result<()> {
    let expected_status = if answer.accept {
        DevicePairingResultStatus::Paired
    } else {
        DevicePairingResultStatus::RejectedByHandler
    };

    let manager = get_bluetooth_manager();
    let status = manager.confirm_device_pairing(&id, answer).await?;

    if status != expected_status {
        return Err(
            format!("Pairing action was not successful! Current status: {status:?}").into(),
        );
    }
    Ok(())
}

#[tauri::command(async)]
pub fn disconnect_bluetooth_device(id: String) -> Result<()> {
    let manager = get_bluetooth_manager();
    manager.disconnect_device(&id)
}

#[tauri::command(async)]
pub async fn forget_bluetooth_device(id: String) -> Result<()> {
    let manager = get_bluetooth_manager();
    let device = if let Some(classic) = manager.devices.get(&id, |d| d.raw.clone()) {
        classic.DeviceInformation()?
    } else if let Some(le) = manager.le_devices.get(&id, |d| d.raw.clone()) {
        le.DeviceInformation()?
    } else {
        return Ok(());
    };

    let status = device.Pairing()?.UnpairAsync()?.await?.Status()?;
    if status == DeviceUnpairingResultStatus::AccessDenied
        || status == DeviceUnpairingResultStatus::Failed
    {
        return Err("Unpair was not succesfull!".into());
    }
    Ok(())
}
