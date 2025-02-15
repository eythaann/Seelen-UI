use crate::{
    modules::bluetooth::{BluetoothEvent, BluetoothManager, BLUETOOTH_MANAGER},
    trace_lock,
};

pub fn register_bluetooth_events() {
    log::trace!("Register for bloetooth events!");

    let _manager = trace_lock!(BLUETOOTH_MANAGER);

    BluetoothManager::subscribe(|event| match event {
        BluetoothEvent::BluetoothDevicesChanged() => {
            let manager = trace_lock!(BLUETOOTH_MANAGER);
            log::trace!("Device list: {:?}", manager.known_items.clone());
        }
        BluetoothEvent::BluetoothDiscoveredDevicesChanged() => {
            let manager = trace_lock!(BLUETOOTH_MANAGER);
            log::trace!("Discovered: {:?}", manager.discovered_items.clone());
        }
    });
}
