use std::sync::{Arc, LazyLock};

use arc_swap::ArcSwapOption;
use seelen_core::system_state::{
    BluetoothDevice as SerializableBluetoothDevice, DevicePairingAnswer, DevicePairingNeededAction,
};
use tokio::sync::mpsc;
use windows::{
    Devices::{
        Bluetooth::{BluetoothDevice, BluetoothLEDevice},
        Enumeration::{
            DeviceInformation, DeviceInformationCustomPairing, DevicePairingKinds,
            DevicePairingProtectionLevel, DevicePairingRequestedEventArgs, DevicePairingResult,
            DevicePairingResultStatus,
        },
    },
    Foundation::{Deferral, IAsyncOperation, TypedEventHandler},
    Security::Credentials::PasswordCredential,
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager, get_tokio_handle, log_error,
    utils::lock_free::SyncHashMap,
    windows_api::{DeviceEnumerator, DeviceEvent, DeviceId},
};

use super::{
    classic::BluetoothDeviceWrapper, low_energy::BluetoothLEDeviceWrapper, BluetoothDeviceType,
    BluetoothManagerEvent,
};

// Pairing configuration constants
const PAIRING_REQUEST_TIMEOUT_SECS: u64 = 10;
const PAIRING_CONFIRMATION_MAX_RETRIES: u32 = 20;
const PAIRING_CONFIRMATION_RETRY_INTERVAL_MS: u64 = 500;

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

    // Pairing state
    pending_pair_requests: SyncHashMap<DeviceId, PendingPairRequest>,
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
            pending_pair_requests: SyncHashMap::new(),
        }
    }

    fn initialize(&mut self) -> Result<()> {
        // Self subscription
        let eid = Self::subscribe(|event| Self::instance().on_event(&event).log_error());
        Self::set_event_handler_priority(&eid, 1);

        // Initialize Bluetooth classic devices enumerator
        let classic_enumerator =
            DeviceEnumerator::new(BluetoothDevice::GetDeviceSelector()?.to_string(), |event| {
                handle_device_event(event, BluetoothDeviceType::Classic)
            })?;

        // Start enumeration and get initial Bluetooth classic devices
        let classic_devices = classic_enumerator.start_blocking()?;
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
        let le_devices = le_enumerator.start_blocking()?;
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
            BluetoothManagerEvent::DeviceAdded(id, device_type) => match device_type {
                BluetoothDeviceType::Classic => {
                    let wrapper = BluetoothDeviceWrapper::create(id)?;
                    self.devices.upsert(id.clone(), wrapper);
                }
                BluetoothDeviceType::LowEnergy => {
                    let wrapper = BluetoothLEDeviceWrapper::create(id)?;
                    self.le_devices.upsert(id.clone(), wrapper);
                }
            },
            BluetoothManagerEvent::DeviceUpdated(id, device_type) => match device_type {
                BluetoothDeviceType::Classic => {
                    self.devices.get(id, |device| {
                        device.refresh_state().log_error();
                    });
                }
                BluetoothDeviceType::LowEnergy => {
                    self.le_devices.get(id, |device| {
                        device.refresh_state().log_error();
                    });
                }
            },
            BluetoothManagerEvent::DeviceRemoved(id, device_type) => {
                self.pending_pair_requests.remove(id);
                match device_type {
                    BluetoothDeviceType::Classic => {
                        self.devices.remove(id);
                    }
                    BluetoothDeviceType::LowEnergy => {
                        self.le_devices.remove(id);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_classic_devices(&self) -> Vec<SerializableBluetoothDevice> {
        let mut devices = Vec::new();
        self.devices.for_each(|(_id, wrapper)| {
            devices.push(wrapper.state.clone());
        });
        devices
    }

    pub fn get_le_devices(&self) -> Vec<SerializableBluetoothDevice> {
        let mut devices = Vec::new();
        self.le_devices.for_each(|(_id, wrapper)| {
            devices.push(wrapper.state.clone());
        });
        devices
    }

    pub fn get_all_devices(&self) -> Vec<SerializableBluetoothDevice> {
        let mut devices = self.get_classic_devices();
        devices.extend(self.get_le_devices());
        devices
    }

    /// Start scanning for unpaired Bluetooth devices
    /// Uses GetDeviceSelectorFromPairingState(false) which automatically turns on system scanning
    pub fn start_scanning(&self) -> Result<()> {
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
    pub fn stop_scanning(&self) -> Result<()> {
        // Discovery enumerators are automatically stopped via Drop trait
        self.discovery_classic_enumerator.store(None);
        self.discovery_le_enumerator.store(None);
        Ok(())
    }

    /// Prepares a device for pairing by setting up event handlers and creating the pending request.
    /// Returns the mpsc receiver, pair handler, and protection level needed for pairing.
    fn prepare_pair_device(
        &self,
        device_id: &str,
    ) -> Result<(
        mpsc::Receiver<Result<DevicePairingNeededAction>>,
        DeviceInformationCustomPairing,
        DevicePairingProtectionLevel,
    )> {
        let device = DeviceInformation::CreateFromIdAsync(&device_id.into())?.get()?;
        let pairing = device.Pairing()?;

        if pairing.IsPaired()? {
            return Err("Device is already paired".into());
        }

        if !pairing.CanPair()? {
            return Err("Device cannot be paired".into());
        }

        let protection_level = pairing.ProtectionLevel()?;
        let pair_handler = pairing.Custom()?;

        let (tx, rx) = mpsc::channel::<Result<DevicePairingNeededAction>>(1);

        // Setup the pairing requested event handler
        let device_id_clone = device_id.to_string();
        let event_token =
            pair_handler.PairingRequested(&TypedEventHandler::new(move |_sender, args| {
                log::trace!("Pairing requested for device {}", device_id_clone);
                let result = Self::on_pair_request(args, device_id_clone.clone());
                let tx = tx.clone();
                get_tokio_handle().spawn(async move {
                    if let Err(e) = tx.send(result).await {
                        log::error!("Failed to send pairing result: {:?}", e);
                    }
                });
                Ok(())
            }))?;

        // Create initial pending request (will be updated by callback and after pairing)
        self.pending_pair_requests.upsert(
            device_id.to_string(),
            PendingPairRequest {
                handler: pair_handler.clone(),
                event_token,
                action: DevicePairingNeededAction::None,
                async_operation: None,
                request: None,
                deferral: None,
            },
        );

        Ok((rx, pair_handler, protection_level))
    }

    /// Starts the pairing process with the given device and waits for the action required.
    /// Returns the action needed from the user.
    async fn start_pairing(
        &self,
        device_id: &str,
        mut rx: mpsc::Receiver<Result<DevicePairingNeededAction>>,
        pair_handler: DeviceInformationCustomPairing,
        protection_level: DevicePairingProtectionLevel,
    ) -> Result<DevicePairingNeededAction> {
        log::trace!("Starting pairing for device {}", device_id);

        // Start the pairing async operation (but don't await it yet to avoid deadlock)
        // The operation will call the PairingRequested callback, which will send us the action
        let pair_async_op = pair_handler.PairWithProtectionLevelAsync(
            DevicePairingKinds::ConfirmOnly
                | DevicePairingKinds::DisplayPin
                | DevicePairingKinds::ProvidePin
                | DevicePairingKinds::ConfirmPinMatch
                | DevicePairingKinds::ProvidePasswordCredential
                | DevicePairingKinds::ProvideAddress,
            protection_level,
        )?;

        // Wait for the callback to determine what action is needed
        // This must happen BEFORE we await the pairing result to avoid deadlock
        // (the callback creates a Deferral that pauses the pairing operation)
        let action = tokio::time::timeout(
            std::time::Duration::from_secs(PAIRING_REQUEST_TIMEOUT_SECS),
            rx.recv(),
        )
        .await
        .map_err(|_| {
            format!(
                "Pairing request timed out after {} seconds",
                PAIRING_REQUEST_TIMEOUT_SECS
            )
        })?
        .ok_or("Pairing channel closed unexpectedly")??;

        // If no valid action is needed, pairing cannot proceed
        if action == DevicePairingNeededAction::None {
            return Err("Device pairing requires unsupported action".into());
        }

        // Store the async operation for later (will be awaited after user confirmation)
        self.pending_pair_requests.get(device_id, |pending| {
            pending.async_operation = Some(pair_async_op);
        });

        Ok(action)
    }

    /// Initiates pairing with a device and returns the action required from the user.
    /// The pending pairing state is stored internally until confirmed or cancelled.
    pub async fn request_pair_device(&self, device_id: &str) -> Result<DevicePairingNeededAction> {
        // Prepare the device for pairing
        let (rx, pair_handler, protection_level) = self.prepare_pair_device(device_id)?;

        // Start pairing and handle cleanup on failure
        match self
            .start_pairing(device_id, rx, pair_handler, protection_level)
            .await
        {
            Ok(action) => Ok(action),
            Err(e) => {
                // Clean up pending request if pairing failed
                self.pending_pair_requests.remove(device_id);
                Err(e)
            }
        }
    }

    /// Confirms or rejects a pending pairing request with user input.
    /// Completes the pairing process and returns the final status.
    pub async fn confirm_device_pairing(
        &self,
        device_id: &str,
        answer: DevicePairingAnswer,
    ) -> Result<DevicePairingResultStatus> {
        let Some(mut pending) = self.pending_pair_requests.remove(device_id) else {
            return Err(format!("No pending pairing request for device {device_id}").into());
        };

        let event_args = pending.request.as_ref().ok_or("Pairing args are null")?;

        // Extract the async operation before dropping pending
        let async_op = pending
            .async_operation
            .take()
            .ok_or("Pairing async operation is null")?;

        // Apply the user's answer to the pairing request
        if answer.accept {
            if let Some(pin) = answer.pin {
                event_args.AcceptWithPin(&pin.into())?;
            } else if let (Some(username), Some(password)) = (answer.username, answer.password) {
                let credential = PasswordCredential::CreatePasswordCredential(
                    &"".into(),
                    &username.into(),
                    &password.into(),
                )?;
                event_args.AcceptWithPasswordCredential(&credential)?;
            } else if let Some(address) = answer.address {
                event_args.AcceptWithAddress(&address.into())?;
            } else {
                event_args.Accept()?;
            }
        }

        // Drop the pending request, which completes the deferral and allows pairing to proceed
        drop(pending);

        // NOW we can await the pairing result (after deferral.Complete() was called in Drop)
        let result = async_op.await?;
        log::trace!("Pairing result for device {}: {:?}", device_id, result);

        // Wait for pairing to complete (for DisplayPin/ConfirmPinMatch, the other device must confirm too)
        if answer.accept {
            let mut paired = false;
            for i in 0..PAIRING_CONFIRMATION_MAX_RETRIES {
                if result.Status()? == DevicePairingResultStatus::Paired {
                    log::info!(
                        "Pairing confirmed for device {} after {} attempts",
                        device_id,
                        i + 1
                    );
                    paired = true;
                    break;
                }

                tokio::time::sleep(std::time::Duration::from_millis(
                    PAIRING_CONFIRMATION_RETRY_INTERVAL_MS,
                ))
                .await;
            }

            if !paired {
                log::warn!(
                    "Pairing polling completed for device {} without confirmation. Cancelling pairing.",
                    device_id
                );
            }
        }

        Ok(result.Status()?)
    }

    /// Handles the pairing requested callback from Windows.
    /// Determines what action is needed and updates the pending request.
    fn on_pair_request(
        request: &Option<DevicePairingRequestedEventArgs>,
        device_id: String,
    ) -> Result<DevicePairingNeededAction> {
        let Some(request) = request else {
            return Err(format!("Pairing args are null for device {}", device_id).into());
        };

        let kind = request.PairingKind()?;
        log::trace!("Pairing kind for device {}: {:?}", device_id, kind);

        // Determine what action is needed from the user based on pairing kind
        let action = match kind {
            DevicePairingKinds::None => DevicePairingNeededAction::None,
            DevicePairingKinds::ConfirmOnly => DevicePairingNeededAction::ConfirmOnly,
            DevicePairingKinds::DisplayPin => {
                let pin = request.Pin()?.to_string();
                DevicePairingNeededAction::DisplayPin { pin }
            }
            DevicePairingKinds::ProvidePin => DevicePairingNeededAction::ProvidePin,
            DevicePairingKinds::ConfirmPinMatch => {
                let pin = request.Pin()?.to_string();
                DevicePairingNeededAction::ConfirmPinMatch { pin }
            }
            DevicePairingKinds::ProvidePasswordCredential => {
                DevicePairingNeededAction::ProvidePasswordCredential
            }
            DevicePairingKinds::ProvideAddress => DevicePairingNeededAction::ProvideAddress,
            _ => {
                log::warn!("Unsupported pairing kind for device {device_id}: {kind:?}");
                DevicePairingNeededAction::None
            }
        };

        if action != DevicePairingNeededAction::None {
            Self::instance()
                .pending_pair_requests
                .get(&device_id, |pending| {
                    pending.action = action.clone();
                    pending.request = Some(request.clone());
                    // The deferral makes the pairing operation wait until user confirmation
                    pending.deferral = request.GetDeferral().ok();
                });
        }

        Ok(action)
    }

    /// Disconnects a paired device without unpairing it.
    ///
    /// Todo: this is not working.
    pub fn disconnect_device(&self, device_id: &str) -> Result<()> {
        if self
            .devices
            .get(device_id, |device| {
                device.disconnect().log_error();
            })
            .is_some()
        {
            return Ok(());
        }

        if let Some(device) = self.le_devices.remove(device_id) {
            device.close()?;
            return Ok(());
        }

        Err(format!("Device not found: {}", device_id).into())
    }

    pub fn release(&mut self) {
        // Device enumerators are automatically stopped via Drop trait
        self.devices.clear();
        self.le_devices.clear();
        log_error!(self.stop_scanning());
    }
}

pub struct PendingPairRequest {
    /// Custom pairing interface for the device
    pub handler: DeviceInformationCustomPairing,
    /// Event handler token to remove on cleanup
    pub event_token: i64,
    /// Action required from the user
    pub action: DevicePairingNeededAction,
    /// Pairing async operation (to be awaited after user confirmation)
    pub async_operation: Option<IAsyncOperation<DevicePairingResult>>,
    /// Event arguments from the pairing callback
    pub request: Option<DevicePairingRequestedEventArgs>,
    /// Deferral to control async pairing flow (present if user input is needed)
    pub deferral: Option<Deferral>,
}

impl Drop for PendingPairRequest {
    fn drop(&mut self) {
        if let Some(deferral) = &self.deferral {
            let _ = deferral.Complete();
        }
        let _ = self.handler.RemovePairingRequested(self.event_token);
    }
}
