use seelen_core::system_state::BluetoothDevice as SerializableBluetoothDevice;
use windows::Devices::Bluetooth::BluetoothLEDevice;

use crate::error::Result;

pub struct BluetoothLEDeviceWrapper {
    pub(super) id: String,
    pub(super) raw: BluetoothLEDevice,
    pub(super) state: SerializableBluetoothDevice,
}

impl BluetoothLEDeviceWrapper {
    pub fn create(device_id: &str) -> Result<Self> {
        let device = BluetoothLEDevice::FromIdAsync(&device_id.into())?.get()?;
        let state = Self::to_serializable(device_id, &device)?;

        Ok(Self {
            id: device_id.to_string(),
            raw: device,
            state,
        })
    }

    pub fn to_serializable(
        id: &str,
        device: &BluetoothLEDevice,
    ) -> Result<SerializableBluetoothDevice> {
        use seelen_core::system_state::enums::{BluetoothMajorClass, BluetoothMinorClass};
        use windows::Devices::Bluetooth::BluetoothConnectionStatus;

        let pairing_state = device.DeviceInformation()?.Pairing()?;

        Ok(SerializableBluetoothDevice {
            id: id.to_owned(),
            name: device.Name()?.to_string(),
            address: device.BluetoothAddress()?,
            major_service_classes: Vec::new(),
            major_class: BluetoothMajorClass::Uncategorized,
            minor_class: BluetoothMinorClass::Uncategorized { unused: 0 },
            appearance: Some(device.Appearance()?.RawValue()?.into()),
            connected: device.ConnectionStatus()? == BluetoothConnectionStatus::Connected,
            paired: pairing_state.IsPaired()?,
            can_pair: pairing_state.CanPair()?,
            is_low_energy: true,
        })
    }
}
