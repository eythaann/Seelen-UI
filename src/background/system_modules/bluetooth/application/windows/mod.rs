mod classic;
mod low_energy;
mod state;

use seelen_core::system_state::{BluetoothDevice, DevicePairingAnswer, DevicePairingNeededAction};
use windows::Devices::Enumeration::{DevicePairingResultStatus, DeviceUnpairingResultStatus};

use crate::{
    error::Result,
    system_modules::bluetooth::domain::{BluetoothEvent, BluetoothSystem},
};

use self::state::BluetoothState;

/// Facade over [`BluetoothState`], the singleton owning the device watchers.
pub struct WindowsBluetooth {
    subscription: Option<String>,
}

impl BluetoothSystem for WindowsBluetooth {
    fn create() -> Self {
        Self { subscription: None }
    }

    fn init<F>(&mut self, on_event: F) -> crate::error::Result<()>
    where
        F: Fn(BluetoothEvent) + Send + Sync + 'static,
    {
        // the raw events are applied to the state first (priority subscriber)
        BluetoothState::instance();
        self.subscription = Some(BluetoothState::subscribe(move |_event| {
            on_event(BluetoothEvent::DevicesChanged)
        }));
        Ok(())
    }

    fn devices(&self) -> Vec<BluetoothDevice> {
        BluetoothState::instance().get_all_devices()
    }

    fn start_scanning(&self) -> Result<()> {
        BluetoothState::instance().start_scanning()
    }

    fn stop_scanning(&self) -> Result<()> {
        BluetoothState::instance().stop_scanning()
    }

    fn restart_scanning_if_active(&self) -> Result<()> {
        BluetoothState::instance().restart_scanning_if_active()
    }

    async fn request_pair(&self, id: String) -> Result<DevicePairingNeededAction> {
        BluetoothState::instance().request_pair_device(&id).await
    }

    async fn confirm_pairing(&self, id: String, answer: DevicePairingAnswer) -> Result<()> {
        let expected_status = if answer.accept {
            DevicePairingResultStatus::Paired
        } else {
            DevicePairingResultStatus::RejectedByHandler
        };
        let status = BluetoothState::instance()
            .confirm_device_pairing(&id, answer)
            .await?;
        if status != expected_status {
            return Err(
                format!("Pairing action was not successful! Current status: {status:?}").into(),
            );
        }
        Ok(())
    }

    fn connect(&self, id: &str) -> Result<()> {
        BluetoothState::instance().connect_device(id)
    }

    fn disconnect(&self, id: &str) -> Result<()> {
        BluetoothState::instance().disconnect_device(id)
    }

    async fn forget(&self, id: String) -> Result<()> {
        let state = BluetoothState::instance();
        let device = if let Some(classic) = state.devices.get(&id, |d| d.raw.clone()) {
            classic.DeviceInformation()?
        } else if let Some(le) = state.le_devices.get(&id, |d| d.raw.clone()) {
            le.DeviceInformation()?
        } else {
            return Ok(());
        };

        let status = device.Pairing()?.UnpairAsync()?.await?.Status()?;
        if status == DeviceUnpairingResultStatus::AccessDenied
            || status == DeviceUnpairingResultStatus::Failed
        {
            return Err("Unpair was not succesfull!".into());
        }
        Ok(())
    }
}

impl Drop for WindowsBluetooth {
    fn drop(&mut self) {
        if let Some(id) = self.subscription.take() {
            BluetoothState::unsubscribe(&id);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BluetoothDeviceType {
    Classic,
    LowEnergy,
}

/// Raw notifications of the device watchers, applied to [`BluetoothState`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
enum RawBluetoothEvent {
    DeviceAdded(String, BluetoothDeviceType),
    DeviceUpdated(String, BluetoothDeviceType),
    DeviceRemoved(String, BluetoothDeviceType),
}
