use seelen_core::system_state::BluetoothDevice as SerializableBluetoothDevice;
use windows::{Devices::Bluetooth::BluetoothDevice, Foundation::TypedEventHandler};

use crate::{
    error::{Result, ResultLogExt},
    modules::radios::bluetooth::{
        manager::BluetoothManager, BluetoothDeviceType, BluetoothManagerEvent,
    },
};

pub struct BluetoothDeviceWrapper {
    pub(super) id: String,
    pub(super) raw: BluetoothDevice,
    pub(super) state: SerializableBluetoothDevice,

    name_changed_token: i64,
    connection_status_changed_token: i64,
}

impl BluetoothDeviceWrapper {
    pub fn create(device_id: &str) -> Result<Self> {
        let device = BluetoothDevice::FromIdAsync(&device_id.into())?.get()?;

        let id = device_id.to_string();
        let name_changed_token =
            device.NameChanged(&TypedEventHandler::new(move |_src, _args| {
                BluetoothManager::send(BluetoothManagerEvent::DeviceUpdated(
                    id.clone(),
                    BluetoothDeviceType::Classic,
                ));
                Ok(())
            }))?;

        let id = device_id.to_string();
        let connection_status_changed_token =
            device.ConnectionStatusChanged(&TypedEventHandler::new(move |_src, _args| {
                BluetoothManager::send(BluetoothManagerEvent::DeviceUpdated(
                    id.clone(),
                    BluetoothDeviceType::Classic,
                ));
                Ok(())
            }))?;

        Ok(Self {
            id: device_id.to_string(),
            state: Self::to_serializable(device_id, &device)?,
            raw: device,
            name_changed_token,
            connection_status_changed_token,
        })
    }

    pub fn refresh_state(&mut self) -> Result<()> {
        self.state = Self::to_serializable(&self.id, &self.raw)?;
        Ok(())
    }

    pub fn disconnect(&self) -> Result<()> {
        // For Classic Bluetooth devices, Windows doesn't provide a direct disconnect API
        // The recommended approach is to close the device and let Windows handle the timeout
        // According to Microsoft docs, Windows will disconnect after 1 second if no references exist

        // Close the device object - Windows will disconnect when there are no active references
        self.raw.Close()?;

        // Note: For Classic Bluetooth, if this doesn't work, the only reliable way
        // is to unpair the device using DeviceInformation.Pairing().UnpairAsync()
        Ok(())
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

        let is_connected = device.ConnectionStatus()? == BluetoothConnectionStatus::Connected;
        let is_paired = pairing_state.IsPaired()?;

        Ok(SerializableBluetoothDevice {
            id: id.to_owned(),
            name: device.Name()?.to_string(),
            address: device.BluetoothAddress()?,
            major_service_classes,
            major_class,
            minor_class,
            appearance: None,
            connected: is_connected,
            paired: is_paired,
            can_pair: pairing_state.CanPair()?,
            can_disconnect: false, // TODO: this will be false until get a way to realize the disconnection without unpairing.
            is_low_energy: false,
        })
    }
}

impl Drop for BluetoothDeviceWrapper {
    fn drop(&mut self) {
        self.raw
            .RemoveNameChanged(self.name_changed_token)
            .log_error();
        self.raw
            .RemoveConnectionStatusChanged(self.connection_status_changed_token)
            .log_error();
    }
}
