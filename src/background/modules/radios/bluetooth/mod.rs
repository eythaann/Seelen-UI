mod classic;
pub mod handlers;
mod low_energy;
mod manager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothDeviceType {
    Classic,
    LowEnergy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothManagerEvent {
    DeviceAdded(String, BluetoothDeviceType),
    DeviceUpdated(String, BluetoothDeviceType),
    DeviceRemoved(String, BluetoothDeviceType),
}
