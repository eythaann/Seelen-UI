use seelen_core::system_state::{RadioDevice, RadioDeviceKind};
use windows::{
    Devices::Radios::{Radio, RadioKind, RadioState},
    Foundation::TypedEventHandler,
};

use crate::{error::Result, modules::radios::manager::RadioManager, windows_api::DeviceEvent};

pub struct SluRadioDevice {
    pub id: String,
    pub raw: Radio,
    pub cache: RadioDevice,
    state_changed_token: i64,
}

impl SluRadioDevice {
    pub fn create(device_id: &str) -> Result<SluRadioDevice> {
        let radio = Radio::FromIdAsync(&device_id.into())?.get()?;

        let id = device_id.to_string();
        let state_changed_token = radio.StateChanged(&TypedEventHandler::new(
            move |sender: &Option<Radio>, _args: &Option<windows_core::IInspectable>| {
                // Get the state OUTSIDE the lock to avoid deadlock
                // The Windows API call (State()) can trigger re-entrant events
                if let Some(sender) = sender {
                    let is_enabled = sender.State().is_ok_and(|s| s == RadioState::On);
                    // Now update the cache with the lock (fast operation)
                    RadioManager::instance().radios.get(&id, |r| {
                        r.cache.is_enabled = is_enabled;
                    });
                    RadioManager::send(DeviceEvent::Updated(id.clone()));
                }
                Ok(())
            },
        ))?;

        Ok(SluRadioDevice {
            id: device_id.to_string(),
            cache: Self::to_serializable(device_id, &radio)?,
            raw: radio,
            state_changed_token,
        })
    }

    pub fn to_serializable(id: &str, radio: &Radio) -> Result<RadioDevice> {
        let kind = match radio.Kind()? {
            RadioKind::WiFi => RadioDeviceKind::WiFi,
            RadioKind::MobileBroadband => RadioDeviceKind::MobileBroadband,
            RadioKind::Bluetooth => RadioDeviceKind::Bluetooth,
            RadioKind::FM => RadioDeviceKind::FM,
            _ => RadioDeviceKind::Other,
        };

        Ok(RadioDevice {
            id: id.to_owned(),
            name: radio.Name()?.to_string(),
            kind,
            is_enabled: radio.State().is_ok_and(|s| s == RadioState::On),
        })
    }
}

impl Drop for SluRadioDevice {
    fn drop(&mut self) {
        let _ = self.raw.RemoveStateChanged(self.state_changed_token);
    }
}
