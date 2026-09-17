use crossbeam_channel::{Receiver, Sender, bounded};
use parking_lot::Mutex;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use windows::{
    Devices::Enumeration::{DeviceInformation, DeviceInformationUpdate, DeviceWatcher},
    Foundation::{IPropertyValue, PropertyType, TypedEventHandler},
    Win32::Devices::DeviceAndDriverInstallation::CM_LOCATE_DEVNODE_FLAGS,
};
use windows_core::Interface;

use crate::error::Result;

pub type DeviceId = String;

/// Events emitted by the device watcher
#[derive(Debug, Clone)]
pub enum DeviceEvent {
    Added(DeviceId),
    Updated(DeviceId),
    Removed(DeviceId),
}

type DeviceChangeCallback = Arc<dyn Fn(DeviceEvent) + Send + Sync + 'static>;

/// Thread-safe device watcher that enumerates and monitors devices using AQS (Advanced Query Syntax)
///
/// # Architecture
/// - During initial enumeration (before `start()` completes), `Added` events are captured but NOT propagated
/// - After enumeration completes, all events (Added/Updated/Removed) are propagated to the callback
/// - This prevents duplicate notifications for devices that exist at startup
#[allow(dead_code)]
pub struct DeviceEnumerator {
    watcher: DeviceWatcher,
    devices: Arc<Mutex<Vec<DeviceInformation>>>,
    enumeration_tx: Sender<()>,
    enumeration_rx: Receiver<()>,
    enumeration_completed: Arc<AtomicBool>,
    callback: DeviceChangeCallback,
}

impl DeviceEnumerator {
    /// Creates a new device enumerator with the specified AQS query string and callback
    ///
    /// # Parameters
    /// - `query`: AQS filter string to specify which devices to monitor
    /// - `callback`: Callback invoked for device events AFTER initial enumeration completes
    ///
    /// # Examples of AQS queries:
    /// - Bluetooth: `System.Devices.Aep.ProtocolId:="{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}"`
    /// - Network: `System.Devices.InterfaceClassGuid:="{cac88484-7515-4c03-82e6-71a87abac361}"`
    /// - Audio: `System.Devices.InterfaceClassGuid:="{2eef81be-33fa-4800-9670-1cd474972c3f}"`
    ///
    /// # Example
    /// ```no_run
    /// let enumerator = DeviceEnumerator::new(
    ///     "System.Devices.Aep.ProtocolId:=\"{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}\"",
    ///     |event| {
    ///         match event {
    ///             DeviceEvent::Added(id) => println!("New device: {}", id),
    ///             DeviceEvent::Updated(id) => println!("Updated: {}", id),
    ///             DeviceEvent::Removed(id) => println!("Removed: {}", id),
    ///         }
    ///     }
    /// )?;
    /// ```
    pub fn new<F>(query: impl Into<String>, callback: F) -> Result<Self>
    where
        F: Fn(DeviceEvent) + Send + Sync + 'static,
    {
        let query: String = query.into();
        let devices: Arc<Mutex<Vec<DeviceInformation>>> = Arc::new(Mutex::new(Vec::new()));
        let (enumeration_tx, enumeration_rx) = bounded(1);
        let enumeration_completed = Arc::new(AtomicBool::new(false));
        let callback: DeviceChangeCallback = Arc::new(callback) as DeviceChangeCallback;

        // Create the device watcher with the AQS filter
        let watcher = DeviceInformation::CreateWatcherAqsFilter(&query.into())?;

        // Setup Added event handler
        {
            let devices = Arc::clone(&devices);
            let callback = callback.clone();
            let enumeration_completed = Arc::clone(&enumeration_completed);
            let handler = TypedEventHandler::new(
                move |_: windows_core::Ref<DeviceWatcher>,
                      info: windows_core::Ref<DeviceInformation>| {
                    if let Some(info) = info.as_ref() {
                        // Always add device to our internal list
                        devices.lock().push(info.clone());

                        // Only notify callback if enumeration has completed
                        // (to avoid notifying for devices that existed at startup)
                        if enumeration_completed.load(Ordering::Acquire)
                            && let Ok(id) = info.Id()
                        {
                            callback(DeviceEvent::Added(id.to_string()));
                        }
                    }
                    Ok(())
                },
            );
            watcher.Added(&handler)?;
        }

        // Setup Updated event handler
        {
            let callback = callback.clone();
            let handler = TypedEventHandler::new(
                move |_: windows_core::Ref<DeviceWatcher>,
                      update: windows_core::Ref<DeviceInformationUpdate>| {
                    if let Some(update) = update.as_ref()
                        && let Ok(id) = update.Id()
                    {
                        callback(DeviceEvent::Updated(id.to_string()));
                    }
                    Ok(())
                },
            );
            watcher.Updated(&handler)?;
        }

        // Setup Removed event handler
        {
            let devices = Arc::clone(&devices);
            let callback = callback.clone();
            let handler = TypedEventHandler::new(
                move |_: windows_core::Ref<DeviceWatcher>,
                      update: windows_core::Ref<DeviceInformationUpdate>| {
                    if let Some(update) = update.as_ref()
                        && let Ok(id) = update.Id()
                    {
                        let id_str = id.to_string();

                        // Remove device from our internal list
                        devices
                            .lock()
                            .retain(|dev| dev.Id().map(|dev_id| dev_id != id_str).unwrap_or(true));

                        // Notify callback
                        callback(DeviceEvent::Removed(id_str));
                    }
                    Ok(())
                },
            );
            watcher.Removed(&handler)?;
        }

        // Setup EnumerationCompleted event handler
        {
            let tx = enumeration_tx.clone();
            let enumeration_completed = Arc::clone(&enumeration_completed);
            let handler = TypedEventHandler::new(
                move |_: windows_core::Ref<DeviceWatcher>,
                      _: windows_core::Ref<windows_core::IInspectable>| {
                    // Mark enumeration as completed FIRST
                    // This allows subsequent Added events to be propagated to the callback
                    enumeration_completed.store(true, Ordering::Release);

                    // Then signal that initial enumeration is complete
                    let _ = tx.send(());
                    Ok(())
                },
            );
            watcher.EnumerationCompleted(&handler)?;
        }

        Ok(Self {
            watcher,
            devices,
            enumeration_tx,
            enumeration_rx,
            enumeration_completed,
            callback,
        })
    }

    /// Starts the device watcher and waits for the initial enumeration to complete
    /// Returns a list of all devices found during the initial enumeration
    ///
    /// This method blocks until the initial enumeration is complete, ensuring that
    /// all existing devices are discovered before returning.
    pub fn start_blocking(&self) -> Result<Vec<DeviceInformation>> {
        // Start the watcher
        self.watcher.Start()?;

        // Wait for the initial enumeration to complete
        self.enumeration_rx
            .recv()
            .map_err(|_| windows::core::Error::from_hresult(windows::Win32::Foundation::E_FAIL))?;

        // Return a clone of all discovered devices
        Ok(self.devices.lock().clone())
    }

    pub fn start(&self) -> Result<()> {
        self.enumeration_completed.store(true, Ordering::Release);
        self.watcher.Start()?;
        Ok(())
    }
}

impl Drop for DeviceEnumerator {
    fn drop(&mut self) {
        let _ = self.watcher.Stop();
    }
}

#[allow(dead_code)]
pub fn get_string_property(
    info: &DeviceInformation,
    name: &str,
) -> windows::core::Result<Option<String>> {
    let value = info.Properties()?.Lookup(&name.into())?;

    let property = value.cast::<IPropertyValue>()?;

    if property.Type()? == PropertyType::String {
        return Ok(Some(property.GetString()?.to_string()));
    }

    Ok(None)
}

use windows::{
    Win32::Devices::DeviceAndDriverInstallation::{
        CM_Get_DevNode_PropertyW, CM_Get_Device_Interface_PropertyW, CM_Locate_DevNodeW,
        CM_MapCrToWin32Err, CONFIGRET, CR_BUFFER_SMALL, CR_NO_SUCH_VALUE, CR_SUCCESS,
    },
    Win32::Devices::Properties::{
        DEVPKEY_Device_DeviceDesc, DEVPKEY_Device_FriendlyName, DEVPKEY_Device_InstanceId,
        DEVPROPTYPE,
    },
    Win32::Foundation::{DEVPROPKEY, ERROR_GEN_FAILURE},
};

use super::string_utils::WindowsString;

fn cr_to_error(cr: CONFIGRET) -> windows::core::Error {
    let win32_err = unsafe { CM_MapCrToWin32Err(cr, ERROR_GEN_FAILURE.0) };
    windows::core::Error::from_hresult(windows::core::HRESULT::from_win32(win32_err))
}

#[allow(dead_code)]
/// Reads a string property of a device node, given its devinst handle and property key.
/// Returns `Ok(None)` when the device has no value set for that property.
fn get_devnode_string_property(devinst: u32, key: &DEVPROPKEY) -> Result<Option<String>> {
    let mut property_type = DEVPROPTYPE::default();
    let mut buffer_size = 0u32;

    // First call: obtain required buffer size.
    let cr = unsafe {
        CM_Get_DevNode_PropertyW(devinst, key, &mut property_type, None, &mut buffer_size, 0)
    };

    if cr == CR_NO_SUCH_VALUE {
        return Ok(None);
    }
    if cr != CR_SUCCESS && cr != CR_BUFFER_SMALL {
        return Err(cr_to_error(cr).into());
    }

    let mut buffer = WindowsString::new_to_fill(buffer_size.div_ceil(2) as usize);

    let cr = unsafe {
        CM_Get_DevNode_PropertyW(
            devinst,
            key,
            &mut property_type,
            Some(buffer.as_mut_slice().as_mut_ptr() as *mut u8),
            &mut buffer_size,
            0,
        )
    };

    if cr != CR_SUCCESS {
        return Err(cr_to_error(cr).into());
    }

    Ok(Some(buffer.to_string()))
}

#[allow(dead_code)]
/// Reads the instance id (`DEVPKEY_Device_InstanceId`) of the device owning a device interface path.
pub fn get_instance_id_from_interface(interface_path: &str) -> Result<String> {
    let path = WindowsString::from_str(interface_path);

    let mut property_type = DEVPROPTYPE::default();
    let mut buffer_size = 0u32;

    let cr = unsafe {
        CM_Get_Device_Interface_PropertyW(
            path.as_pcwstr(),
            &DEVPKEY_Device_InstanceId,
            &mut property_type,
            None,
            &mut buffer_size,
            0,
        )
    };

    if cr != CR_SUCCESS && cr != CR_BUFFER_SMALL {
        return Err(cr_to_error(cr).into());
    }

    let mut buffer = WindowsString::new_to_fill(buffer_size.div_ceil(2) as usize);

    let cr = unsafe {
        CM_Get_Device_Interface_PropertyW(
            path.as_pcwstr(),
            &DEVPKEY_Device_InstanceId,
            &mut property_type,
            Some(buffer.as_mut_slice().as_mut_ptr() as *mut u8),
            &mut buffer_size,
            0,
        )
    };

    if cr != CR_SUCCESS {
        return Err(cr_to_error(cr).into());
    }

    Ok(buffer.to_string())
}

#[allow(dead_code)]
/// Obtains the hardware device name shown in Device Manager: the device's friendly name,
/// falling back to its device description when no friendly name was set.
pub fn get_device_friendly_name(instance_id: &str) -> Result<String> {
    let instance_id = WindowsString::from_str(instance_id);

    let mut devinst: u32 = 0;

    let cr = unsafe {
        CM_Locate_DevNodeW(
            &mut devinst,
            instance_id.as_pcwstr(),
            CM_LOCATE_DEVNODE_FLAGS(0),
        )
    };

    if cr != CR_SUCCESS {
        return Err(cr_to_error(cr).into());
    }

    if let Some(name) = get_devnode_string_property(devinst, &DEVPKEY_Device_FriendlyName)? {
        return Ok(name);
    }

    Ok(get_devnode_string_property(devinst, &DEVPKEY_Device_DeviceDesc)?.unwrap_or_default())
}
