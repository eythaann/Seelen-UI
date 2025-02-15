use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use itertools::Itertools;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::Devices::Bluetooth::BluetoothDevice;
use windows::Devices::Enumeration::{DeviceInformation, DeviceInformationUpdate, DeviceWatcher};
use windows::Foundation::{EventRegistrationToken, TypedEventHandler};
use windows_core::HSTRING;

use crate::utils::spawn_named_thread;
use crate::windows_api::WindowsApi;
use crate::{event_manager, log_error, trace_lock};

use crate::error_handler::Result;

use super::domain::BluetoothDeviceInfo;

lazy_static! {
    pub static ref BLUETOOTH_MANAGER: Arc<Mutex<BluetoothManager>> = Arc::new(Mutex::new(
        BluetoothManager::new().expect("Failed to create bluetooth manager")
    ));
    static ref SIGNALED_DISCOVER_END: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BluetoothEvent {
    BluetoothDevicesChanged(),
    BluetoothDiscoveredDevicesChanged(),
}

#[derive(Debug)]
pub struct BluetoothManager {
    pub known_items: HashMap<String, BluetoothDeviceInfo>,
    pub discovered_items: HashMap<u64, BluetoothDeviceInfo>,

    enumeration_completed: bool,

    // COM object & handlers
    watcher: Option<DeviceWatcher>,
    device_added_handler: TypedEventHandler<DeviceWatcher, DeviceInformation>,
    device_added_registration: Option<EventRegistrationToken>,
    device_updated_handler: TypedEventHandler<DeviceWatcher, DeviceInformationUpdate>,
    device_updated_registration: Option<EventRegistrationToken>,
    device_removed_handler: TypedEventHandler<DeviceWatcher, DeviceInformationUpdate>,
    device_removed_registration: Option<EventRegistrationToken>,
    device_enumeration_completed_handler:
        TypedEventHandler<DeviceWatcher, windows_core::IInspectable>,
    device_enumeration_completed_registration: Option<EventRegistrationToken>,

    known_items_registration_handlers: HashMap<String, Vec<EventRegistrationToken>>,
}

unsafe impl Send for BluetoothManager {}
unsafe impl Send for BluetoothEvent {}

event_manager!(BluetoothManager, BluetoothEvent);

impl BluetoothManager {
    pub fn new() -> Result<Self> {
        let mut instance = Self {
            known_items: HashMap::new(),
            discovered_items: HashMap::new(),
            enumeration_completed: false,
            watcher: None,
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
        };

        // if let Ok(items) = WindowsApi::enum_bluetooth_device(false) {
        //     for item in items.into_iter().map_into::<BluetoothDeviceInfo>() {
        //         instance.known_items.insert(item.id.clone(), item);
        //     }
        // }

        _ = instance.register_for_bt_devices();

        Ok(instance)
    }

    pub fn register_for_bt_devices(&mut self) -> Result<()> {
        if self.watcher.is_some() {
            return Ok(());
        }

        let watcher =
            DeviceInformation::CreateWatcherAqsFilter(&BluetoothDevice::GetDeviceSelector()?)?;
        self.device_added_registration = watcher.Added(&self.device_added_handler).ok();
        self.device_updated_registration = watcher.Updated(&self.device_updated_handler).ok();
        self.device_removed_registration = watcher.Removed(&self.device_removed_handler).ok();
        self.device_enumeration_completed_registration = watcher
            .EnumerationCompleted(&self.device_enumeration_completed_handler)
            .ok();

        watcher.Start()?;
        self.watcher = Some(watcher);

        Ok(())
    }

    fn add_device(&mut self, id: String, device: BluetoothDevice) -> Result<()> {
        let info: BluetoothDeviceInfo = device.into();

        if let Some(ref inner) = info.inner {
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
        }

        if self.discovered_items.remove(&info.address).is_some() {
            log_error!(Self::event_tx().send(BluetoothEvent::BluetoothDiscoveredDevicesChanged()));
        }

        self.known_items.insert(id, info); //update or insert

        if self.enumeration_completed {
            log_error!(Self::event_tx().send(BluetoothEvent::BluetoothDevicesChanged()));
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
                }
            }
        }

        if self.enumeration_completed {
            log_error!(Self::event_tx().send(BluetoothEvent::BluetoothDevicesChanged()));
        }
        Ok(())
    }
    fn set_enumeration_completed(&mut self) -> Result<()> {
        self.enumeration_completed = true;

        log_error!(Self::event_tx().send(BluetoothEvent::BluetoothDevicesChanged()));

        _ = self.discover();

        Ok(())
    }

    pub fn update_device(&mut self, device: &BluetoothDevice) -> Result<()> {
        let info: BluetoothDeviceInfo = device.clone().into();
        self.known_items.insert(info.id.clone(), info); //update or insert

        if self.enumeration_completed {
            log_error!(Self::event_tx().send(BluetoothEvent::BluetoothDevicesChanged()));
        }

        Ok(())
    }

    pub fn discover(&mut self) -> Result<()> {
        spawn_named_thread("BLUETOOTH_MANAGER_DISCOVERY", move || {
            SIGNALED_DISCOVER_END.store(false, Ordering::Release);

            loop {
                if SIGNALED_DISCOVER_END.load(Ordering::Acquire) {
                    log::trace!("Discovery terminating!");
                    SIGNALED_DISCOVER_END.store(false, Ordering::Release);
                    break;
                }

                let result = BluetoothManager::on_discover_loop();
                log_error!(&result);
                if result.is_err() {
                    thread::sleep(Duration::from_secs(10));
                }
            }
        })?;

        Ok(())
    }

    fn on_discover_loop() -> Result<()> {
        let devices = WindowsApi::enum_bluetooth_device(true)?;

        let mut manager = trace_lock!(BLUETOOTH_MANAGER);
        manager.discovered_items.clear();

        for item in devices
            .into_iter()
            .filter(|item| !(item.fConnected.as_bool() || item.fRemembered.as_bool()))
            .map_into::<BluetoothDeviceInfo>()
        {
            manager.discovered_items.insert(item.address, item);
        }

        log_error!(Self::event_tx().send(BluetoothEvent::BluetoothDiscoveredDevicesChanged()));

        Ok(())
    }

    pub fn stop_discovery(&mut self) -> Result<()> {
        if !SIGNALED_DISCOVER_END.load(Ordering::Acquire) {
            SIGNALED_DISCOVER_END.store(true, Ordering::Release);
        }

        Ok(())
    }
}

//Proxy event handlers for device added or removed
impl BluetoothManager {
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
        let device = BluetoothDevice::FromIdAsync(&id)?.get()?;
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
