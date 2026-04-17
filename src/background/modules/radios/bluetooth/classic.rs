use seelen_core::system_state::BluetoothDevice as SerializableBluetoothDevice;
use windows::{
    Devices::Bluetooth::BluetoothDevice,
    Foundation::TypedEventHandler,
    Win32::Devices::Bluetooth::{BLUETOOTH_SERVICE_DISABLE, BLUETOOTH_SERVICE_ENABLE},
};

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
        let device = BluetoothDevice::FromIdAsync(&device_id.into())?.join()?;

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

    pub fn to_serializable(
        id: &str,
        device: &BluetoothDevice,
    ) -> Result<SerializableBluetoothDevice> {
        use seelen_core::system_state::enums::{BluetoothMajorClass, BluetoothMajorServiceClass};
        use windows::Devices::Bluetooth::BluetoothConnectionStatus;

        let class = device.ClassOfDevice()?;
        let pairing_state = device.DeviceInformation()?.Pairing()?;
        let class_value = class.RawValue()?;
        let (major_service_classes, major_class, minor_class) =
            SerializableBluetoothDevice::get_parts_of_class(class_value);

        let is_paired = pairing_state.IsPaired()?;
        let is_connected = device.ConnectionStatus()? == BluetoothConnectionStatus::Connected;
        let is_audio = major_service_classes.contains(&BluetoothMajorServiceClass::Audio)
            || major_service_classes.contains(&BluetoothMajorServiceClass::LowEnergyAudio)
            || major_class == BluetoothMajorClass::AudioVideo;

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
            can_disconnect: is_connected && is_audio,
            can_connect: is_paired && !is_connected && is_audio,
            is_low_energy: false,
        })
    }
}

/// Disconnects a paired Classic BT device by sending IOCTL_BTH_DISCONNECT_DEVICE to the radio.
///
/// This is the only documented way to physically drop the ACL link without unpairing.
/// Reference: https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/bthioctl/ni-bthioctl-ioctl_bth_disconnect_device
pub(super) fn disconnect_via_ioctl(address: u64) -> crate::error::Result<()> {
    use std::mem::size_of;
    use windows::Win32::Devices::Bluetooth::{
        BluetoothFindFirstRadio, BluetoothFindRadioClose, BLUETOOTH_FIND_RADIO_PARAMS,
    };
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::IO::DeviceIoControl;

    // CTL_CODE(FILE_DEVICE_BLUETOOTH=0x41, 0x03, METHOD_BUFFERED=0, FILE_ANY_ACCESS=0)
    const IOCTL_BTH_DISCONNECT_DEVICE: u32 = 0x0041000C;

    let params = BLUETOOTH_FIND_RADIO_PARAMS {
        dwSize: size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32,
    };
    let mut h_radio = HANDLE::default();
    let h_find = unsafe { BluetoothFindFirstRadio(&params, &mut h_radio)? };

    let mut bytes_returned = 0u32;
    let result = unsafe {
        DeviceIoControl(
            h_radio,
            IOCTL_BTH_DISCONNECT_DEVICE,
            Some(std::ptr::addr_of!(address).cast()),
            size_of::<u64>() as u32,
            None,
            0,
            Some(&mut bytes_returned),
            None,
        )
    };

    unsafe { BluetoothFindRadioClose(h_find)? };
    result?;
    Ok(())
}

/// Reconnects a paired Classic BT audio device by toggling its audio service profiles.
/// The service must be disabled first — if it is already marked enabled, calling enable
/// again is a no-op. The disable→enable cycle forces Windows to initiate a new connection.
pub(super) fn connect_via_service_state(address: u64) -> crate::error::Result<()> {
    use std::mem::{size_of, zeroed};
    use windows::core::GUID;
    use windows::Win32::Devices::Bluetooth::{BluetoothSetServiceState, BLUETOOTH_DEVICE_INFO};

    // Bluetooth SIG service class UUIDs — 16-bit short UUIDs expanded to 128-bit
    // using the BT base UUID: 00000000-0000-1000-8000-00805F9B34FB.
    // Reference: https://www.bluetooth.com/specifications/assigned-numbers/ (Service Class UUIDs)

    // A2DP Sink (0x110B): receives audio — headphones, speakers, car audio, etc.
    const A2DP_SINK: GUID = GUID::from_u128(0x0000110b_0000_1000_8000_00805f9b34fb);

    // HFP Hands-Free (0x111E): call audio channel for headsets with microphone.
    const HFP: GUID = GUID::from_u128(0x0000111e_0000_1000_8000_00805f9b34fb);

    let mut info: BLUETOOTH_DEVICE_INFO = unsafe { zeroed() };
    info.dwSize = size_of::<BLUETOOTH_DEVICE_INFO>() as u32;
    info.Address.Anonymous.ullLong = address;

    for guid in &[A2DP_SINK, HFP] {
        // None = use the first available local radio (documented NULL behaviour)
        unsafe {
            let _ = BluetoothSetServiceState(None, &info, guid, BLUETOOTH_SERVICE_DISABLE);
            // std::thread::sleep(std::time::Duration::from_millis(100));
            let _ = BluetoothSetServiceState(None, &info, guid, BLUETOOTH_SERVICE_ENABLE);
        }
    }
    Ok(())
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
