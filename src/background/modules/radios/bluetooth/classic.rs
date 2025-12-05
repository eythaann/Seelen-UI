use seelen_core::system_state::BluetoothDevice as SerializableBluetoothDevice;
use windows::Devices::Bluetooth::BluetoothDevice;

use crate::error::Result;

pub struct BluetoothDeviceWrapper {
    pub(super) id: String,
    pub(super) raw: BluetoothDevice,
    pub(super) state: SerializableBluetoothDevice,
}

impl BluetoothDeviceWrapper {
    pub fn create(device_id: &str) -> Result<Self> {
        let device = BluetoothDevice::FromIdAsync(&device_id.into())?.get()?;
        let state = Self::to_serializable(device_id, &device)?;

        Ok(Self {
            id: device_id.to_string(),
            raw: device,
            state,
        })
    }

    pub fn to_serializable(
        id: &str,
        device: &BluetoothDevice,
    ) -> Result<SerializableBluetoothDevice> {
        use windows::Devices::Bluetooth::BluetoothConnectionStatus;

        let class = device.ClassOfDevice()?;
        let pairing_state = device.DeviceInformation()?.Pairing()?;
        let class_value = class.RawValue()?;
        let (major_service_classes, major_class, minor_class) =
            SerializableBluetoothDevice::get_parts_of_class(class_value);

        Ok(SerializableBluetoothDevice {
            id: id.to_owned(),
            name: device.Name()?.to_string(),
            address: device.BluetoothAddress()?,
            major_service_classes,
            major_class,
            minor_class,
            appearance: None,
            connected: device.ConnectionStatus()? == BluetoothConnectionStatus::Connected,
            paired: pairing_state.IsPaired()?,
            can_pair: pairing_state.CanPair()?,
            is_low_energy: false,
        })
    }
}
