use seelen_core::system_state::BluetoothDevice as SerializableBluetoothDevice;
use windows::{Devices::Bluetooth::BluetoothLEDevice, Foundation::TypedEventHandler};

use crate::{
    error::{Result, ResultLogExt},
    modules::radios::bluetooth::{
        manager::BluetoothManager, BluetoothDeviceType, BluetoothManagerEvent,
    },
};

pub struct BluetoothLEDeviceWrapper {
    pub(super) id: String,
    pub(super) raw: BluetoothLEDevice,
    pub(super) state: SerializableBluetoothDevice,

    name_changed_token: i64,
    connection_status_changed_token: i64,
}

impl BluetoothLEDeviceWrapper {
    pub fn create(device_id: &str) -> Result<Self> {
        let device = BluetoothLEDevice::FromIdAsync(&device_id.into())?.get()?;

        let id = device_id.to_string();
        let name_changed_token =
            device.NameChanged(&TypedEventHandler::new(move |_src, _args| {
                BluetoothManager::send(BluetoothManagerEvent::DeviceUpdated(
                    id.clone(),
                    BluetoothDeviceType::LowEnergy,
                ));
                Ok(())
            }))?;

        let id = device_id.to_string();
        let connection_status_changed_token =
            device.ConnectionStatusChanged(&TypedEventHandler::new(move |_src, _args| {
                BluetoothManager::send(BluetoothManagerEvent::DeviceUpdated(
                    id.clone(),
                    BluetoothDeviceType::LowEnergy,
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

    pub fn close(self) -> Result<()> {
        self.raw.RemoveNameChanged(self.name_changed_token)?;
        self.raw
            .RemoveConnectionStatusChanged(self.connection_status_changed_token)?;

        // For BLE devices, we need to close all GATT sessions first, then services
        // According to Microsoft docs, closing sessions is what actually triggers disconnect
        let services = self.raw.GetGattServicesAsync()?.get()?.Services()?;
        for service in services {
            if let Ok(session) = service.Session() {
                session.Close()?;
            }
            service.Close()?;
        }

        // Finally close the device object itself
        self.raw.Close()?;
        Ok(())
    }

    pub fn to_serializable(
        id: &str,
        device: &BluetoothLEDevice,
    ) -> Result<SerializableBluetoothDevice> {
        use seelen_core::system_state::enums::{BluetoothMajorClass, BluetoothMinorClass};
        use windows::Devices::Bluetooth::BluetoothConnectionStatus;

        let pairing_state = device.DeviceInformation()?.Pairing()?;

        let is_connected = device.ConnectionStatus()? == BluetoothConnectionStatus::Connected;
        let is_paired = pairing_state.IsPaired()?;

        Ok(SerializableBluetoothDevice {
            id: id.to_owned(),
            name: device.Name()?.to_string(),
            address: device.BluetoothAddress()?,
            major_service_classes: Vec::new(),
            major_class: BluetoothMajorClass::Uncategorized,
            minor_class: BluetoothMinorClass::Uncategorized { unused: 0 },
            appearance: Some(device.Appearance()?.RawValue()?.into()),
            connected: is_connected,
            paired: is_paired,
            can_pair: pairing_state.CanPair()?,
            can_disconnect: false, // TODO: this will be false until get a way to realize the disconnection without unpairing.
            is_low_energy: true,
        })
    }
}

impl Drop for BluetoothLEDeviceWrapper {
    fn drop(&mut self) {
        self.raw
            .RemoveNameChanged(self.name_changed_token)
            .log_error();
        self.raw
            .RemoveConnectionStatusChanged(self.connection_status_changed_token)
            .log_error();
    }
}
