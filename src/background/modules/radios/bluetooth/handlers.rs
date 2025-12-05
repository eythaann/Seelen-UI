use seelen_core::system_state::BluetoothDevice;

use crate::modules::radios::bluetooth::manager::BluetoothManager;

fn get_bluetooth_manager() -> &'static BluetoothManager {
    BluetoothManager::instance()
}

#[tauri::command(async)]
pub fn get_bluetooth_devices() -> Vec<BluetoothDevice> {
    get_bluetooth_manager().get_all_devices()
}
