pub mod bluetooth_pair_manager;

use std::collections::HashMap;
use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::Devices::Bluetooth::{BluetoothDevice, BluetoothLEDevice};
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationUpdate, DeviceWatcher};
use windows::Devices::Radios::{Radio, RadioKind, RadioState};
use windows::Foundation::TypedEventHandler;
use windows_core::HSTRING;

use crate::{event_manager, hstring, log_error, trace_lock};

use crate::error_handler::Result;

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
    RadioStateChanged(bool),
}

#[derive(Debug)]
pub struct BluetoothManager {
    pub known_items: HashMap<String, BluetoothDeviceInfo>,
    pub discovered_items: HashMap<u64, BluetoothDeviceInfo>,

    enumeration_completed: bool,

    // COM radio object & handlers
    radios: HashMap<String, Radio>,
    radio_watcher: Option<DeviceWatcher>,
    radio_added_handler: TypedEventHandler<DeviceWatcher, DeviceInformation>,
    radio_added_registration: Option<i64>,
    radio_updated_handler: TypedEventHandler<DeviceWatcher, DeviceInformationUpdate>,
    radio_updated_registration: Option<i64>,
    radio_removed_handler: TypedEventHandler<DeviceWatcher, DeviceInformationUpdate>,
    radio_removed_registration: Option<i64>,

    radio_state_changed_handler: TypedEventHandler<Radio, windows_core::IInspectable>,
    radio_state_changed_registration_handlers: HashMap<String, i64>,

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
    log_error!(manager.release_radios());
}

impl BluetoothManager {
    pub fn new() -> Result<Self> {
        let mut instance = Self {
            known_items: HashMap::new(),
            discovered_items: HashMap::new(),
            radios: HashMap::new(),
            radio_watcher: None,
            radio_added_handler: TypedEventHandler::new(BluetoothManager::on_radio_added),
            radio_added_registration: None,
            radio_updated_handler: TypedEventHandler::new(BluetoothManager::on_radio_updated),
            radio_updated_registration: None,
            radio_removed_handler: TypedEventHandler::new(BluetoothManager::on_radio_removed),
            radio_removed_registration: None,
            radio_state_changed_handler: TypedEventHandler::new(
                BluetoothManager::on_radio_state_changed,
            ),
            radio_state_changed_registration_handlers: HashMap::new(),
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

        instance.prepare_radios()?;

        Ok(instance)
    }

    fn prepare_radios(&mut self) -> Result<()> {
        if self.radio_watcher.is_some() {
            return Ok(());
        }

        let watcher = DeviceInformation::CreateWatcherAqsFilter(&Radio::GetDeviceSelector()?)?;
        self.radio_added_registration = watcher.Added(&self.radio_added_handler).ok();
        self.radio_updated_registration = watcher.Updated(&self.radio_updated_handler).ok();
        self.radio_removed_registration = watcher.Removed(&self.radio_removed_handler).ok();
        watcher.Start()?;
        self.radio_watcher = Some(watcher);
        Ok(())
    }

    pub fn release_radios(&mut self) -> Result<()> {
        if let Some(watcher) = &self.radio_watcher {
            if let Some(token) = self.radio_added_registration {
                watcher.RemoveAdded(token)?;
                self.device_added_registration = None;
            }
            if let Some(token) = self.radio_updated_registration {
                watcher.RemoveUpdated(token)?;
                self.device_updated_registration = None;
            }
            if let Some(token) = self.radio_removed_registration {
                watcher.RemoveRemoved(token)?;
                self.device_removed_registration = None;
            }
            watcher.Stop()?;

            self.radio_watcher = None;

            for (key, _) in self.known_items.clone() {
                if let Some(radio) = self.radios.remove(&key) {
                    if let Some(registration) =
                        self.radio_state_changed_registration_handlers.remove(&key)
                    {
                        radio.RemoveStateChanged(registration)?;
                    }
                }
            }

            self.radios.clear();
        }
        Ok(())
    }

    pub fn get_radio_state(&self) -> Result<bool> {
        Ok(self
            .radios
            .iter()
            .any(|(_, value)| value.State().unwrap_or(RadioState::On) == RadioState::On))
    }

    pub fn set_radio_state(&self, enable: bool) -> Result<()> {
        for value in self.radios.values() {
            value
                .SetStateAsync(if enable {
                    RadioState::On
                } else {
                    RadioState::Off
                })?
                .get()?;
        }
        Ok(())
    }

    pub fn register_for_bt_devices(&mut self) -> Result<()> {
        if self.device_watcher.is_some() {
            return Ok(());
        }
        if self
            .radios
            .iter()
            .all(|(_, value)| value.State().unwrap_or(RadioState::Off) == RadioState::Off)
        {
            return Err(
                "Bluetooth enumeration can not be started because no radio can be activated!"
                    .into(),
            );
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
        if self
            .radios
            .iter()
            .all(|(_, value)| value.State().unwrap_or(RadioState::Off) == RadioState::Off)
        {
            return Err(
                "Bluetooth enumeration can not be started because no radio can be activated!"
                    .into(),
            );
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

    pub(super) fn on_radio_added(
        _sender: &Option<DeviceWatcher>,
        args: &Option<DeviceInformation>,
    ) -> windows_core::Result<()> {
        if let Some(device) = args {
            let id = device.Id()?;
            let radio = Radio::FromIdAsync(&id)?.get()?;
            if radio.Kind()? == RadioKind::Bluetooth {
                let mut manager = trace_lock!(BLUETOOTH_MANAGER);
                let registration = radio.StateChanged(&manager.radio_state_changed_handler)?;
                manager
                    .radio_state_changed_registration_handlers
                    .insert(id.to_string(), registration);
                manager.radios.insert(id.to_string(), radio);
                if let Ok(state) = manager.get_radio_state() {
                    log_error!(Self::event_tx().send(BluetoothEvent::RadioStateChanged(state)));
                }
                log_error!(manager.register_for_bt_devices());
            }
        }
        Ok(())
    }

    pub(super) fn on_radio_updated(
        _sender: &Option<DeviceWatcher>,
        args: &Option<DeviceInformationUpdate>,
    ) -> windows_core::Result<()> {
        if let Some(device) = args {
            let id = device.Id()?;
            let radio = Radio::FromIdAsync(&id)?.get()?;
            if radio.Kind()? == RadioKind::Bluetooth {
                {
                    let mut manager = trace_lock!(BLUETOOTH_MANAGER);
                    manager.radios.insert(id.to_string(), radio);
                }
                log_error!(Self::handle_radio_state_change());
            }
        }
        Ok(())
    }

    pub(super) fn on_radio_removed(
        _sender: &Option<DeviceWatcher>,
        args: &Option<DeviceInformationUpdate>,
    ) -> windows_core::Result<()> {
        if let Some(device) = args {
            let id = device.Id()?;
            let radio = Radio::FromIdAsync(&id)?.get()?;
            if radio.Kind()? == RadioKind::Bluetooth {
                {
                    let mut manager = trace_lock!(BLUETOOTH_MANAGER);
                    if let Some(token) = manager
                        .radio_state_changed_registration_handlers
                        .remove(&id.to_string())
                    {
                        radio.RemoveStateChanged(token)?;
                    }
                    manager.radios.remove(&id.to_string());
                }
                log_error!(Self::handle_radio_state_change());
            }
        }
        Ok(())
    }

    pub(super) fn on_radio_state_changed(
        _sender: &Option<Radio>,
        _args: &Option<windows_core::IInspectable>,
    ) -> windows_core::Result<()> {
        log_error!(Self::handle_radio_state_change());
        Ok(())
    }

    fn handle_radio_state_change() -> Result<()> {
        let mut manager = trace_lock!(BLUETOOTH_MANAGER);
        if let Ok(state) = manager.get_radio_state() {
            manager.enumeration_completed = false;

            if state {
                manager.register_for_bt_devices()?;
            } else {
                manager.deregister_for_bt_devices()?;
                manager.stop_discovery()?;
            }

            log_error!(Self::event_tx().send(BluetoothEvent::RadioStateChanged(state)));
        }
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
