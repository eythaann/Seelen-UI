use std::sync::LazyLock;

use seelen_core::system_state::{RadioDevice, RadioDeviceKind};
use windows::Devices::Radios::{Radio, RadioAccessStatus, RadioState};

use super::device::SluRadioDevice;
use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::SyncHashMap,
    windows_api::{DeviceEnumerator, DeviceEvent, DeviceId},
};

pub struct RadiosState {
    pub radios: SyncHashMap<DeviceId, SluRadioDevice>,
    device_enumerator: Option<DeviceEnumerator>,
}

unsafe impl Send for RadiosState {}
unsafe impl Sync for RadiosState {}

event_manager!(RadiosState, RawRadioEvent);

/// Raw notifications of the radios watchers.
#[derive(Debug, Clone)]
pub enum RawRadioEvent {
    /// A radio was added, removed or changed its state.
    Changed,
    /// A radio was turned on.
    Enabled(RadioDeviceKind),
}

#[allow(dead_code)]
impl RadiosState {
    fn create() -> Self {
        Self {
            radios: SyncHashMap::new(),
            device_enumerator: None,
        }
    }

    fn initialize(&mut self) -> Result<()> {
        // Create device enumerator with callback
        let enumerator = DeviceEnumerator::new(Radio::GetDeviceSelector()?.to_string(), |event| {
            RadiosState::instance().on_event(&event).log_error();
            RadiosState::send(RawRadioEvent::Changed);
        })?;

        // Start enumeration (blocks until initial enumeration completes)
        let devices = enumerator.start_blocking()?;

        // Map DeviceInformation to (Radio, i64) with state change handler
        let radios: Result<Vec<SluRadioDevice>> = devices
            .iter()
            .map(|device| {
                let id = device.Id()?.to_string();
                SluRadioDevice::create(&id)
            })
            .collect();

        let radios_map: std::collections::HashMap<DeviceId, SluRadioDevice> = radios?
            .into_iter()
            .map(|radio| (radio.id.clone(), radio))
            .collect();
        self.radios = SyncHashMap::from(radios_map);
        self.device_enumerator = Some(enumerator);
        Ok(())
    }

    pub fn instance() -> &'static Self {
        static RADIO_MANAGER_INSTANCE: LazyLock<RadiosState> = LazyLock::new(|| {
            let mut m = RadiosState::create();
            m.initialize().log_error();
            m
        });
        &RADIO_MANAGER_INSTANCE
    }

    pub fn is_enabled(&self, kind: RadioDeviceKind) -> bool {
        self.radios
            .any(|(_id, radio)| radio.cache.kind == kind && radio.cache.is_enabled)
    }

    fn on_event(&self, event: &DeviceEvent) -> Result<()> {
        match event {
            DeviceEvent::Added(id) => {
                let radio = SluRadioDevice::create(id)?;
                self.radios.upsert(id.clone(), radio);
            }
            DeviceEvent::Updated(_id) => {}
            DeviceEvent::Removed(id) => {
                self.radios.remove(id);
            }
        }
        Ok(())
    }

    pub fn get_radios(&self) -> Vec<RadioDevice> {
        let mut radios = Vec::new();
        self.radios
            .for_each(|(_id, radio)| radios.push(radio.cache.clone()));
        radios
    }

    pub fn set_radios_state(&self, kind: RadioDeviceKind, enabled: bool) -> Result<()> {
        if Radio::RequestAccessAsync()?.join()? != RadioAccessStatus::Allowed {
            // todo handle this via UI error.
            return Ok(());
        }

        let mut to_update = Vec::new();
        self.radios.for_each(|(_id, radio)| {
            if radio.cache.kind == kind {
                to_update.push(radio.raw.clone());
            }
        });

        let state = if enabled {
            RadioState::On
        } else {
            RadioState::Off
        };
        for radio in to_update {
            radio.SetStateAsync(state)?.join()?;
        }
        Ok(())
    }
}
