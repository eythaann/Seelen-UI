use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use windows::{
    Devices::Enumeration::{DeviceInformation, DeviceInformationUpdate, DeviceWatcher},
    Foundation::TypedEventHandler,
};

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
                move |_: &Option<DeviceWatcher>, info: &Option<DeviceInformation>| {
                    if let Some(info) = info {
                        // Always add device to our internal list
                        devices.lock().push(info.clone());

                        // Only notify callback if enumeration has completed
                        // (to avoid notifying for devices that existed at startup)
                        if enumeration_completed.load(Ordering::Acquire) {
                            if let Ok(id) = info.Id() {
                                callback(DeviceEvent::Added(id.to_string()));
                            }
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
                move |_: &Option<DeviceWatcher>, update: &Option<DeviceInformationUpdate>| {
                    if let Some(update) = update {
                        if let Ok(id) = update.Id() {
                            callback(DeviceEvent::Updated(id.to_string()));
                        }
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
                move |_: &Option<DeviceWatcher>, update: &Option<DeviceInformationUpdate>| {
                    if let Some(update) = update {
                        if let Ok(id) = update.Id() {
                            let id_str = id.to_string();

                            // Remove device from our internal list
                            devices.lock().retain(|dev| {
                                dev.Id().map(|dev_id| dev_id != id_str).unwrap_or(true)
                            });

                            // Notify callback
                            callback(DeviceEvent::Removed(id_str));
                        }
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
                move |_: &Option<DeviceWatcher>, _: &Option<windows::core::IInspectable>| {
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
