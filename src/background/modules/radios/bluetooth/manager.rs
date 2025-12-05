use std::sync::{Arc, LazyLock};

use arc_swap::ArcSwapOption;
use seelen_core::system_state::BluetoothDevice as SerializableBluetoothDevice;
use windows::Devices::Bluetooth::{BluetoothDevice, BluetoothLEDevice};

use crate::{
    error::Result,
    event_manager, log_error,
    utils::lock_free::SyncHashMap,
    windows_api::{DeviceEnumerator, DeviceEvent, DeviceId},
};

use super::{
    classic::BluetoothDeviceWrapper, low_energy::BluetoothLEDeviceWrapper, BluetoothDeviceType,
    BluetoothManagerEvent,
};

static BLUETOOTH_MANAGER_INSTANCE: LazyLock<BluetoothManager> = LazyLock::new(|| {
    let mut m = BluetoothManager::create();
    log_error!(m.initialize());
    m
});

pub struct BluetoothManager {
    pub devices: SyncHashMap<DeviceId, BluetoothDeviceWrapper>,
    pub le_devices: SyncHashMap<DeviceId, BluetoothLEDeviceWrapper>,

    classic_enumerator: Option<DeviceEnumerator>,
    le_enumerator: Option<DeviceEnumerator>,

    // Discovery/scanning enumerators (unpaired devices)
    discovery_classic_enumerator: ArcSwapOption<DeviceEnumerator>,
    discovery_le_enumerator: ArcSwapOption<DeviceEnumerator>,
}

unsafe impl Send for BluetoothManager {}
unsafe impl Sync for BluetoothManager {}

event_manager!(BluetoothManager, BluetoothManagerEvent);

/// Helper function to map DeviceEvent to BluetoothManagerEvent and handle it
fn handle_device_event(event: DeviceEvent, device_type: BluetoothDeviceType) {
    let bt_event = match event {
        DeviceEvent::Added(id) => BluetoothManagerEvent::DeviceAdded(id, device_type),
        DeviceEvent::Updated(id) => BluetoothManagerEvent::DeviceUpdated(id, device_type),
        DeviceEvent::Removed(id) => BluetoothManagerEvent::DeviceRemoved(id, device_type),
    };
    log_error!(BluetoothManager::instance().on_event(&bt_event));
    BluetoothManager::send(bt_event);
}

#[allow(dead_code)]
impl BluetoothManager {
    fn create() -> Self {
        Self {
            devices: SyncHashMap::new(),
            le_devices: SyncHashMap::new(),
            classic_enumerator: None,
            le_enumerator: None,
            discovery_classic_enumerator: ArcSwapOption::from(None),
            discovery_le_enumerator: ArcSwapOption::from(None),
        }
    }

    fn initialize(&mut self) -> Result<()> {
        // Initialize Bluetooth classic devices enumerator
        let classic_enumerator =
            DeviceEnumerator::new(BluetoothDevice::GetDeviceSelector()?.to_string(), |event| {
                handle_device_event(event, BluetoothDeviceType::Classic)
            })?;

        // Start enumeration and get initial Bluetooth classic devices
        let classic_devices = classic_enumerator.start()?;
        let devices: Result<Vec<BluetoothDeviceWrapper>> = classic_devices
            .iter()
            .map(|device| {
                let id = device.Id()?.to_string();
                BluetoothDeviceWrapper::create(&id)
            })
            .collect();

        let devices_map: std::collections::HashMap<DeviceId, BluetoothDeviceWrapper> = devices?
            .into_iter()
            .map(|wrapper| (wrapper.id.clone(), wrapper))
            .collect();
        self.devices = SyncHashMap::from(devices_map);
        self.classic_enumerator = Some(classic_enumerator);

        // Initialize Bluetooth LE devices enumerator
        let le_enumerator = DeviceEnumerator::new(
            BluetoothLEDevice::GetDeviceSelector()?.to_string(),
            |event| handle_device_event(event, BluetoothDeviceType::LowEnergy),
        )?;

        // Start enumeration and get initial Bluetooth LE devices
        let le_devices = le_enumerator.start()?;
        let devices_le: Result<Vec<BluetoothLEDeviceWrapper>> = le_devices
            .iter()
            .map(|device| {
                let id = device.Id()?.to_string();
                BluetoothLEDeviceWrapper::create(&id)
            })
            .collect();

        let devices_le_map: std::collections::HashMap<DeviceId, BluetoothLEDeviceWrapper> =
            devices_le?
                .into_iter()
                .map(|wrapper| (wrapper.id.clone(), wrapper))
                .collect();
        self.le_devices = SyncHashMap::from(devices_le_map);
        self.le_enumerator = Some(le_enumerator);

        Ok(())
    }

    pub fn instance() -> &'static Self {
        &BLUETOOTH_MANAGER_INSTANCE
    }

    fn on_event(&self, event: &BluetoothManagerEvent) -> Result<()> {
        match event {
            BluetoothManagerEvent::DeviceAdded(id, device_type)
            | BluetoothManagerEvent::DeviceUpdated(id, device_type) => match device_type {
                BluetoothDeviceType::Classic => {
                    let wrapper = BluetoothDeviceWrapper::create(id)?;
                    self.devices.upsert(id.clone(), wrapper);
                }
                BluetoothDeviceType::LowEnergy => {
                    let wrapper = BluetoothLEDeviceWrapper::create(id)?;
                    self.le_devices.upsert(id.clone(), wrapper);
                }
            },
            BluetoothManagerEvent::DeviceRemoved(id, device_type) => match device_type {
                BluetoothDeviceType::Classic => {
                    self.devices.remove(id);
                }
                BluetoothDeviceType::LowEnergy => {
                    self.le_devices.remove(id);
                }
            },
        }
        Ok(())
    }

    pub fn get_classic_devices(&self) -> Vec<SerializableBluetoothDevice> {
        let mut result = Vec::new();
        self.devices.for_each(|(_id, wrapper)| {
            result.push(wrapper.state.clone());
        });
        result
    }

    pub fn get_le_devices(&self) -> Vec<SerializableBluetoothDevice> {
        let mut result = Vec::new();
        self.le_devices.for_each(|(_id, wrapper)| {
            result.push(wrapper.state.clone());
        });
        result
    }

    pub fn get_all_devices(&self) -> Vec<SerializableBluetoothDevice> {
        let mut all_devices = self.get_classic_devices();
        all_devices.extend(self.get_le_devices());
        all_devices
    }

    /// Start scanning for unpaired Bluetooth devices
    /// Uses GetDeviceSelectorFromPairingState(false) which automatically turns on system scanning
    pub fn start_scanning(&mut self) -> Result<()> {
        // If already scanning, do nothing
        if self.discovery_classic_enumerator.load().is_some()
            || self.discovery_le_enumerator.load().is_some()
        {
            return Ok(());
        }

        // Start scanning for unpaired classic Bluetooth devices
        let classic_selector = BluetoothDevice::GetDeviceSelectorFromPairingState(false)?;
        let discovery_classic_enumerator =
            DeviceEnumerator::new(classic_selector.to_string(), |event| {
                handle_device_event(event, BluetoothDeviceType::Classic)
            })?;

        discovery_classic_enumerator.start()?;
        self.discovery_classic_enumerator
            .store(Some(Arc::new(discovery_classic_enumerator)));

        // Start scanning for unpaired Bluetooth LE devices
        let le_selector = BluetoothLEDevice::GetDeviceSelectorFromPairingState(false)?;
        let discovery_le_enumerator = DeviceEnumerator::new(le_selector.to_string(), |event| {
            handle_device_event(event, BluetoothDeviceType::LowEnergy)
        })?;

        discovery_le_enumerator.start()?;
        self.discovery_le_enumerator
            .store(Some(Arc::new(discovery_le_enumerator)));

        Ok(())
    }

    /// Stop scanning for unpaired Bluetooth devices
    pub fn stop_scanning(&mut self) -> Result<()> {
        // Discovery enumerators are automatically stopped via Drop trait
        self.discovery_classic_enumerator.store(None);
        self.discovery_le_enumerator.store(None);
        Ok(())
    }

    pub fn release(&mut self) {
        // Device enumerators are automatically stopped via Drop trait
        self.devices.clear();
        self.le_devices.clear();
        log_error!(self.stop_scanning());
    }
}
