pub mod bluetooth_pair_manager;

use std::collections::HashMap;
use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::Devices::Bluetooth::{BluetoothDevice, BluetoothLEDevice};
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationUpdate, DeviceWatcher};
use windows::Devices::Radios::RadioKind;
use windows::Foundation::TypedEventHandler;
use windows_core::HSTRING;

use crate::modules::shared::radio::RADIO_MANAGER;
use crate::{event_manager, hstring, log_error, trace_lock};

use crate::error::Result;

use super::domain::BluetoothDeviceInfo;

lazy_static! {
    pub static ref BLUETOOTH_MANAGER: Arc<Mutex<BluetoothManager>> = Arc::new(Mutex::new(
        BluetoothManager::new().expect("Failed to create bluetooth manager")
    ));
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum BluetoothEvent {
    DevicesChanged(Vec<BluetoothDeviceInfo>),
    DiscoveredDevicesChanged(Vec<BluetoothDeviceInfo>),
}

#[derive(Debug)]
pub struct BluetoothManager {
    pub known_items: HashMap<String, BluetoothDeviceInfo>,
    pub discovered_items: HashMap<u64, BluetoothDeviceInfo>,

    enumeration_completed: bool,

    // COM device object & handlers
    device_watcher: Option<DeviceWatcher>,
    device_added_handler: TypedEventHandler<DeviceWatcher, DeviceInformation>,
    device_added_registration: Option<i64>,
    device_updated_handler: TypedEventHandler<DeviceWatcher, DeviceInformationUpdate>,
    device_updated_registration: Option<i64>,
    device_removed_handler: TypedEventHandler<DeviceWatcher, DeviceInformationUpdate>,
    device_removed_registration: Option<i64>,
    device_enumeration_completed_handler:
        TypedEventHandler<DeviceWatcher, windows_core::IInspectable>,
    device_enumeration_completed_registration: Option<i64>,

    known_items_registration_handlers: HashMap<String, Vec<i64>>,

    // COM issue query
    inquery_watcher: Option<DeviceWatcher>,
    inquery_added_handler: TypedEventHandler<DeviceWatcher, DeviceInformation>,
    inquery_added_registration: Option<i64>,
    inquery_enumeration_completed_handler:
        TypedEventHandler<DeviceWatcher, windows_core::IInspectable>,
    inquery_enumeration_completed_registration: Option<i64>,
}

unsafe impl Send for BluetoothManager {}
unsafe impl Send for BluetoothEvent {}

event_manager!(BluetoothManager, BluetoothEvent);

pub fn release_bluetooth_events() {
    let mut manager = trace_lock!(BLUETOOTH_MANAGER);
    log_error!(manager.stop_discovery());
    log_error!(manager.deregister_for_bt_devices());
}

impl BluetoothManager {
    pub fn new() -> Result<Self> {
        let instance = Self {
            known_items: HashMap::new(),
            discovered_items: HashMap::new(),
            enumeration_completed: false,
            device_watcher: None,
            device_added_handler: TypedEventHandler::new(BluetoothManager::on_device_added),
            device_added_registration: None,
            device_updated_handler: TypedEventHandler::new(BluetoothManager::on_device_updated),
            device_updated_registration: None,
            device_removed_handler: TypedEventHandler::new(BluetoothManager::on_device_removed),
            device_removed_registration: None,
            device_enumeration_completed_handler: TypedEventHandler::new(
                BluetoothManager::on_enumeration_completed,
            ),
            device_enumeration_completed_registration: None,
            known_items_registration_handlers: HashMap::new(),
            inquery_watcher: None,
            inquery_added_handler: TypedEventHandler::new(
                BluetoothManager::on_discovery_device_added,
            ),
            inquery_added_registration: None,
            inquery_enumeration_completed_handler: TypedEventHandler::new(
                BluetoothManager::on_discovery_completed,
            ),
            inquery_enumeration_completed_registration: None,
        };
        Ok(instance)
    }

    pub fn is_bluetooth_enabled() -> bool {
        trace_lock!(RADIO_MANAGER).is_enabled(RadioKind::Bluetooth)
    }

    pub fn register_for_bt_devices(&mut self) -> Result<()> {
        if self.device_watcher.is_some() {
            return Ok(());
        }

        if !Self::is_bluetooth_enabled() {
            return Err("There's no bluetooth radio enabled!".into());
        }

        // log::trace!("({0}) OR ({1})", BluetoothDevice.GetDeviceSelector(), BluetoothLEDevice.GetDeviceSelector())
        let query_string = hstring!("(System.Devices.DevObjectType:=5 AND System.Devices.Aep.ProtocolId:=\"{E0CBF06C-CD8B-4647-BB8A-263B43F0F974}\" AND (System.Devices.Aep.IsPaired:=System.StructuredQueryType.Boolean#True OR System.Devices.Aep.Bluetooth.IssueInquiry:=System.StructuredQueryType.Boolean#False)) OR (System.Devices.DevObjectType:=5 AND System.Devices.Aep.ProtocolId:=\"{BB7BB05E-5972-42B5-94FC-76EAA7084D49}\" AND (System.Devices.Aep.IsPaired:=System.StructuredQueryType.Boolean#True OR System.Devices.Aep.Bluetooth.IssueInquiry:=System.StructuredQueryType.Boolean#False))");
        let watcher = DeviceInformation::CreateWatcherAqsFilter(query_string)?;
        self.device_added_registration = watcher.Added(&self.device_added_handler).ok();
        self.device_updated_registration = watcher.Updated(&self.device_updated_handler).ok();
        self.device_removed_registration = watcher.Removed(&self.device_removed_handler).ok();
        self.device_enumeration_completed_registration = watcher
            .EnumerationCompleted(&self.device_enumeration_completed_handler)
            .ok();

        watcher.Start()?;
        self.device_watcher = Some(watcher);

        Ok(())
    }

    pub fn deregister_for_bt_devices(&mut self) -> Result<()> {
        if let Some(watcher) = &self.device_watcher {
            if let Some(token) = self.device_added_registration {
                watcher.RemoveAdded(token)?;
                self.device_added_registration = None;
            }
            if let Some(token) = self.device_updated_registration {
                watcher.RemoveUpdated(token)?;
                self.device_updated_registration = None;
            }
            if let Some(token) = self.device_removed_registration {
                watcher.RemoveRemoved(token)?;
                self.device_removed_registration = None;
            }

            if let Some(token) = self.device_enumeration_completed_registration {
                watcher.RemoveEnumerationCompleted(token)?;
                self.device_enumeration_completed_registration = None;
            }
            watcher.Stop()?;
            self.device_watcher = None;

            for (key, _) in self.known_items.clone() {
                self.remove_device(key)?;
            }
        }
        Ok(())
    }

    fn add_device(&mut self, id: String, device: BluetoothDeviceInfo) -> Result<()> {
        if let Some(ref inner) = device.inner {
            self.known_items_registration_handlers.insert(
                id.clone(),
                vec![
                    inner.ConnectionStatusChanged(&TypedEventHandler::new(
                        BluetoothDeviceInfo::on_device_attribute_changed,
                    ))?,
                    inner.NameChanged(&TypedEventHandler::new(
                        BluetoothDeviceInfo::on_device_attribute_changed,
                    ))?,
                ],
            );
        } else if let Some(ref inner) = device.inner_le {
            self.known_items_registration_handlers.insert(
                id.clone(),
                vec![
                    inner.ConnectionStatusChanged(&TypedEventHandler::new(
                        BluetoothDeviceInfo::on_le_device_attribute_changed,
                    ))?,
                    inner.NameChanged(&TypedEventHandler::new(
                        BluetoothDeviceInfo::on_le_device_attribute_changed,
                    ))?,
                ],
            );
        }

        if self.discovered_items.remove(&device.address).is_some() {
            log_error!(
                Self::event_tx().send(BluetoothEvent::DiscoveredDevicesChanged(
                    self.discovered_items.values().cloned().collect()
                ))
            );
        }

        self.known_items.insert(id, device); //update or insert

        if self.enumeration_completed {
            log_error!(Self::event_tx().send(BluetoothEvent::DevicesChanged(
                self.known_items.values().cloned().collect()
            )));
        }
        Ok(())
    }

    fn remove_device(&mut self, key: String) -> Result<()> {
        if let Some(device) = self.known_items.remove(&key) {
            if let Some(mut registrations) = self.known_items_registration_handlers.remove(&key) {
                if let Some(inner) = device.inner {
                    let connection_registration = registrations.pop().unwrap();
                    inner.RemoveConnectionStatusChanged(connection_registration)?;
                    let name_registration = registrations.pop().unwrap();
                    inner.RemoveNameChanged(name_registration)?;
                } else if let Some(inner) = device.inner_le {
                    let connection_registration = registrations.pop().unwrap();
                    inner.RemoveConnectionStatusChanged(connection_registration)?;
                    let name_registration = registrations.pop().unwrap();
                    inner.RemoveNameChanged(name_registration)?;
                }
            }
        }

        if self.enumeration_completed {
            log_error!(Self::event_tx().send(BluetoothEvent::DevicesChanged(
                self.known_items.values().cloned().collect()
            )));
        }
        Ok(())
    }

    fn set_enumeration_completed(&mut self) -> Result<()> {
        self.enumeration_completed = true;
        log_error!(Self::event_tx().send(BluetoothEvent::DevicesChanged(
            self.known_items.values().cloned().collect()
        )));

        Ok(())
    }

    pub fn update_device(&mut self, device: BluetoothDeviceInfo) -> Result<()> {
        self.known_items.insert(device.id.clone(), device); //update or insert

        if self.enumeration_completed {
            log_error!(Self::event_tx().send(BluetoothEvent::DevicesChanged(
                self.known_items.values().cloned().collect()
            )));
        }
        Ok(())
    }

    pub fn discover(&mut self) -> Result<()> {
        if !Self::is_bluetooth_enabled() {
            return Err("There's no bluetooth radio enabled!".into());
        }

        if self.inquery_watcher.is_some() {
            self.stop_discovery()?;
        }

        // A special query string which results the pairable discovered bluetooth & bluetooth LE devices with inquery.
        // log::trace!("({0}) OR ({1})", BluetoothDevice::GetDeviceSelectorFromPairingState(false), BluetoothLEDevice::GetDeviceSelectorFromPairingState(false))
        let search = hstring!("(System.Devices.DevObjectType:=5 AND System.Devices.Aep.ProtocolId:=\"{E0CBF06C-CD8B-4647-BB8A-263B43F0F974}\" AND (System.Devices.Aep.IsPaired:=System.StructuredQueryType.Boolean#False OR System.Devices.Aep.Bluetooth.IssueInquiry:=System.StructuredQueryType.Boolean#True) AND System.Devices.Aep.CanPair:=System.StructuredQueryType.Boolean#True) OR (System.Devices.DevObjectType:=5 AND System.Devices.Aep.ProtocolId:=\"{bb7bb05e-5972-42b5-94fc-76eaa7084d49}\" AND System.Devices.Aep.CanPair:=System.StructuredQueryType.Boolean#True AND System.Devices.Aep.IsPaired:=System.StructuredQueryType.Boolean#False)");
        let watcher = DeviceInformation::CreateWatcherAqsFilter(search)?;
        self.inquery_added_registration = watcher.Added(&self.inquery_added_handler).ok();
        self.inquery_enumeration_completed_registration = watcher
            .EnumerationCompleted(&self.inquery_enumeration_completed_handler)
            .ok();
        watcher.Start()?;

        self.inquery_watcher = Some(watcher);

        Ok(())
    }

    pub fn stop_discovery(&mut self) -> Result<()> {
        if let Some(watcher) = &self.inquery_watcher {
            if let Some(token) = self.inquery_added_registration {
                watcher.RemoveAdded(token)?;
                self.inquery_added_registration = None;
            }
            if let Some(token) = self.inquery_enumeration_completed_registration {
                watcher.RemoveEnumerationCompleted(token)?;
                self.inquery_enumeration_completed_registration = None;
            }

            watcher.Stop()?;
            self.inquery_watcher = None;
        }

        Ok(())
    }
}

//Proxy event handlers for device & radio observation events
impl BluetoothManager {
    pub(super) fn on_discovery_device_added(
        _sender: &Option<DeviceWatcher>,
        args: &Option<DeviceInformation>,
    ) -> windows_core::Result<()> {
        if let Some(info) = args {
            let id = info.Id()?;
            let device: BluetoothDeviceInfo = if id.to_string().starts_with("BluetoothLE#") {
                let device = BluetoothLEDevice::FromIdAsync(&id)?.get()?;
                device.try_into()?
            } else {
                let device = BluetoothDevice::FromIdAsync(&id)?.get()?;
                device.try_into()?
            };
            let mut manager = trace_lock!(BLUETOOTH_MANAGER);
            manager
                .discovered_items
                .insert(device.address, device.clone());
        }

        Ok(())
    }

    pub(super) fn on_discovery_completed(
        _sender: &Option<DeviceWatcher>,
        _args: &Option<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        let mut manager = trace_lock!(BLUETOOTH_MANAGER);
        log_error!(
            Self::event_tx().send(BluetoothEvent::DiscoveredDevicesChanged(
                manager.discovered_items.values().cloned().collect()
            ))
        );
        log_error!(manager.discover());

        Ok(())
    }

    pub(super) fn on_device_added(
        _sender: &Option<DeviceWatcher>,
        args: &Option<DeviceInformation>,
    ) -> windows_core::Result<()> {
        if let Some(device) = args {
            let id = device.Id()?;
            log_error!(BluetoothManager::insert_or_update(id));
        }
        Ok(())
    }

    pub(super) fn on_device_updated(
        _sender: &Option<DeviceWatcher>,
        args: &Option<DeviceInformationUpdate>,
    ) -> windows_core::Result<()> {
        if let Some(device) = args {
            let id = device.Id()?;
            log_error!(BluetoothManager::insert_or_update(id));
        }
        Ok(())
    }

    pub(super) fn on_device_removed(
        _sender: &Option<DeviceWatcher>,
        args: &Option<DeviceInformationUpdate>,
    ) -> windows_core::Result<()> {
        if let Some(device) = args {
            let id = device.Id()?;
            log_error!(BluetoothManager::remove(id));
        }
        Ok(())
    }

    pub(super) fn on_enumeration_completed(
        _sender: &Option<DeviceWatcher>,
        _args: &Option<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        let mut manager = trace_lock!(BLUETOOTH_MANAGER);
        log_error!(manager.set_enumeration_completed());
        Ok(())
    }

    fn insert_or_update(id: HSTRING) -> Result<()> {
        let device: BluetoothDeviceInfo = if id.to_string().starts_with("BluetoothLE#") {
            let device = BluetoothLEDevice::FromIdAsync(&id)?.get()?;
            device.try_into()?
        } else {
            let device = BluetoothDevice::FromIdAsync(&id)?.get()?;
            device.try_into()?
        };
        let mut manager = trace_lock!(BLUETOOTH_MANAGER);
        manager.add_device(id.to_string(), device)?;
        Ok(())
    }
    fn remove(id: HSTRING) -> Result<()> {
        let mut manager = trace_lock!(BLUETOOTH_MANAGER);
        manager.remove_device(id.to_string())?;
        Ok(())
    }
}
