use seelen_core::system_state::BluetoothDevice as SerializableBluetoothDevice;
use windows::{
    Devices::Bluetooth::{BluetoothLEDevice, GenericAttributeProfile::GattSession},
    Foundation::TypedEventHandler,
};

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
    /// Held to keep the BLE connection alive; None when disconnected.
    connection_session: Option<GattSession>,

    name_changed_token: i64,
    connection_status_changed_token: i64,
}

impl BluetoothLEDeviceWrapper {
    pub fn create(device_id: &str) -> Result<Self> {
        let device = BluetoothLEDevice::FromIdAsync(&device_id.into())?.join()?;

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
            connection_session: None,
            name_changed_token,
            connection_status_changed_token,
        })
    }

    pub fn refresh_state(&mut self) -> Result<()> {
        self.state = Self::to_serializable(&self.id, &self.raw)?;
        Ok(())
    }

    /// Connects by creating a GattSession with MaintainConnection=true.
    /// Windows keeps the BLE radio link alive as long as this session is held.
    pub fn connect(&mut self) -> Result<()> {
        let bt_id = self.raw.BluetoothDeviceId()?;
        let session = GattSession::FromDeviceIdAsync(&bt_id)?.join()?;
        session.SetMaintainConnection(true)?;
        self.connection_session = Some(session);
        Ok(())
    }

    /// Disconnects by releasing the GattSession, allowing the OS to drop the link.
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(session) = self.connection_session.take() {
            session.SetMaintainConnection(false).log_error();
            session.Close().log_error();
        }
        Ok(())
    }

    pub fn to_serializable(
        id: &str,
        device: &BluetoothLEDevice,
    ) -> Result<SerializableBluetoothDevice> {
        use seelen_core::system_state::enums::{BluetoothMajorClass, BluetoothMinorClass};
        use windows::Devices::Bluetooth::BluetoothConnectionStatus;

        let pairing_state = device.DeviceInformation()?.Pairing()?;

        let is_paired = pairing_state.IsPaired()?;
        let is_connected = device.ConnectionStatus()? == BluetoothConnectionStatus::Connected;

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
            can_disconnect: false,
            can_connect: is_paired && !is_connected,
            is_low_energy: true,
        })
    }
}

impl Drop for BluetoothLEDeviceWrapper {
    fn drop(&mut self) {
        if let Some(session) = self.connection_session.take() {
            session.SetMaintainConnection(false).log_error();
            session.Close().log_error();
        }
        if self.name_changed_token != 0 {
            self.raw
                .RemoveNameChanged(self.name_changed_token)
                .log_error();
        }
        if self.connection_status_changed_token != 0 {
            self.raw
                .RemoveConnectionStatusChanged(self.connection_status_changed_token)
                .log_error();
        }
    }
}
