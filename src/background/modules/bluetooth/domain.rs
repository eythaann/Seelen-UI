use windows::Devices::Bluetooth::{BluetoothConnectionStatus, BluetoothDevice, BluetoothLEDevice};

use crate::{log_error, modules::bluetooth::BLUETOOTH_MANAGER, trace_lock};

use seelen_core::system_state::{
    enums::{BluetoothMajorClass, BluetoothMajorServiceClass, BluetoothMinorClass},
    low_energy_enums::BLEAppearance,
    BluetoothDevice as SerializableBluetoothDevice,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BluetoothDeviceInfo {
    pub id: String,
    pub name: String,
    pub address: u64,
    pub major_service_classes: Vec<BluetoothMajorServiceClass>,
    pub major_class: BluetoothMajorClass,
    pub minor_class: BluetoothMinorClass,
    pub appearance: Option<BLEAppearance>,
    pub connected: bool,
    pub paired: bool,
    pub can_pair: bool,

    pub inner: Option<BluetoothDevice>,
    pub inner_le: Option<BluetoothLEDevice>,
}

impl From<BluetoothDeviceInfo> for SerializableBluetoothDevice {
    fn from(v: BluetoothDeviceInfo) -> Self {
        SerializableBluetoothDevice {
            id: v.id,
            name: v.name,
            address: v.address,
            major_service_classes: v.major_service_classes,
            major_class: v.major_class,
            minor_class: v.minor_class,
            appearance: v.appearance,
            connected: v.connected,
            paired: v.paired,
            can_pair: v.can_pair,
            is_low_energy: v.inner_le.is_some(),
        }
    }
}

impl TryFrom<BluetoothDevice> for BluetoothDeviceInfo {
    type Error = windows_core::Error;

    fn try_from(bluetooth_device: BluetoothDevice) -> windows_core::Result<Self> {
        let class = bluetooth_device.ClassOfDevice()?;
        let pairing_state = bluetooth_device.DeviceInformation()?.Pairing()?;

        let class = class.RawValue()?;
        let (major_service_classes, major_class, minor_class) =
            SerializableBluetoothDevice::get_parts_of_class(class);

        Ok(Self {
            id: bluetooth_device.BluetoothDeviceId()?.Id()?.to_string(),
            name: bluetooth_device.Name()?.to_string(),
            address: bluetooth_device.BluetoothAddress()?,
            major_service_classes,
            major_class,
            minor_class,
            appearance: None,
            connected: bluetooth_device.ConnectionStatus()? == BluetoothConnectionStatus::Connected,
            paired: pairing_state.IsPaired()?,
            can_pair: pairing_state.CanPair()?,
            inner: Some(bluetooth_device),
            inner_le: None,
        })
    }
}

impl TryFrom<BluetoothLEDevice> for BluetoothDeviceInfo {
    type Error = windows_core::Error;

    fn try_from(bluetooth_device: BluetoothLEDevice) -> windows_core::Result<Self> {
        let pairing_state = bluetooth_device.DeviceInformation()?.Pairing()?;
        Ok(Self {
            id: bluetooth_device.BluetoothDeviceId()?.Id()?.to_string(),
            name: bluetooth_device.Name()?.to_string(),
            address: bluetooth_device.BluetoothAddress()?,
            major_service_classes: Vec::new(),
            major_class: BluetoothMajorClass::Uncategorized,
            minor_class: BluetoothMinorClass::Uncategorized { unused: 0 },
            appearance: Some(bluetooth_device.Appearance()?.RawValue()?.into()),
            connected: bluetooth_device.ConnectionStatus()? == BluetoothConnectionStatus::Connected,
            paired: pairing_state.IsPaired()?,
            can_pair: pairing_state.CanPair()?,
            inner: None,
            inner_le: Some(bluetooth_device),
        })
    }
}

//Proxy event handlers for device attrivute changed
impl BluetoothDeviceInfo {
    pub(super) fn on_device_attribute_changed(
        sender: &Option<BluetoothDevice>,
        _args: &Option<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        if let Some(device) = sender {
            let mut manager = trace_lock!(BLUETOOTH_MANAGER);
            log_error!(manager.update_device(device.clone().try_into()?));
        }

        Ok(())
    }
    pub(super) fn on_le_device_attribute_changed(
        sender: &Option<BluetoothLEDevice>,
        _args: &Option<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        if let Some(device) = sender {
            let mut manager = trace_lock!(BLUETOOTH_MANAGER);
            log_error!(manager.update_device(device.clone().try_into()?));
        }

        Ok(())
    }
}
