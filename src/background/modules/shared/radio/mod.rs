use parking_lot::Mutex;
use windows::{
    Devices::{
        Enumeration::{DeviceInformation, DeviceInformationUpdate, DeviceWatcher},
        Radios::{Radio, RadioKind, RadioState},
    },
    Foundation::TypedEventHandler,
};

use crate::{error::Result, event_manager, log_error, trace_lock};

lazy_static! {
    pub static ref RADIO_MANAGER: Mutex<RadioManager> = Mutex::new(RadioManager::new());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadioManagerEvent {
    Added(String),
    Updated(String),
    Removed(String),
}

pub struct RadioManager {
    pub radios: Vec<(Radio, i64)>,
    watcher: Option<DeviceWatcher>,
    radio_added_handler: (
        TypedEventHandler<DeviceWatcher, DeviceInformation>,
        Option<i64>,
    ),
    radio_updated_handler: (
        TypedEventHandler<DeviceWatcher, DeviceInformationUpdate>,
        Option<i64>,
    ),
    radio_removed_handler: (
        TypedEventHandler<DeviceWatcher, DeviceInformationUpdate>,
        Option<i64>,
    ),
    radio_state_changed_handler: TypedEventHandler<Radio, windows_core::IInspectable>,
}

unsafe impl Send for RadioManager {}

event_manager!(RadioManager, RadioManagerEvent);

#[allow(dead_code)]
impl RadioManager {
    fn new() -> Self {
        Self {
            radios: Vec::new(),
            watcher: None,
            radio_added_handler: (TypedEventHandler::new(on_radio_added), None),
            radio_updated_handler: (TypedEventHandler::new(on_radio_updated), None),
            radio_removed_handler: (TypedEventHandler::new(on_radio_removed), None),
            radio_state_changed_handler: TypedEventHandler::new(on_radio_state_changed),
        }
    }

    pub fn is_enabled(&self, kind: RadioKind) -> bool {
        self.radios.iter().any(|(radio, _)| {
            radio.Kind().is_ok_and(|k| k == kind)
                && radio.State().is_ok_and(|s| s == RadioState::On)
        })
    }

    fn get_all_radios(&mut self) -> Result<Vec<(Radio, i64)>> {
        let mut radios = Vec::new();
        for radio in Radio::GetRadiosAsync()?.get()? {
            let token = radio.StateChanged(&self.radio_state_changed_handler)?;
            radios.push((radio, token));
        }
        Ok(radios)
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.radios = self.get_all_radios()?;

        let watcher = DeviceInformation::CreateWatcherAqsFilter(&Radio::GetDeviceSelector()?)?;
        self.radio_added_handler.1 = watcher.Added(&self.radio_added_handler.0).ok();
        self.radio_updated_handler.1 = watcher.Updated(&self.radio_updated_handler.0).ok();
        self.radio_removed_handler.1 = watcher.Removed(&self.radio_removed_handler.0).ok();
        watcher.Start()?;
        self.watcher = Some(watcher);

        Self::subscribe(|e| log_error!(trace_lock!(RADIO_MANAGER).on_event(e)));
        Ok(())
    }

    fn on_event(&mut self, event: RadioManagerEvent) -> Result<()> {
        match event {
            RadioManagerEvent::Added(id) => {
                let radio = Radio::FromIdAsync(&id.into())?.get()?;
                let token = radio.StateChanged(&self.radio_state_changed_handler)?;
                self.radios.push((radio, token));
            }
            RadioManagerEvent::Updated(_id) => {}
            RadioManagerEvent::Removed(_id) => {
                self.radios.retain(|(radio, _)| {
                    radio
                        .State()
                        .is_ok_and(|state| state != RadioState::Unknown)
                });
            }
        }
        Ok(())
    }

    pub fn turn_on_radios(&self, kind: RadioKind) -> Result<()> {
        for (radio, _) in &self.radios {
            if radio.Kind()? == kind {
                radio.SetStateAsync(RadioState::On)?.get()?;
            }
        }
        Ok(())
    }

    pub fn turn_off_radios(&self, kind: RadioKind) -> Result<()> {
        for (radio, _) in &self.radios {
            if radio.Kind()? == kind {
                radio.SetStateAsync(RadioState::Off)?.get()?;
            }
        }
        Ok(())
    }

    pub fn release(&mut self) {
        if let Some(watcher) = &self.watcher {
            if let Some(token) = self.radio_added_handler.1 {
                log_error!(watcher.RemoveAdded(token));
            }
            if let Some(token) = self.radio_updated_handler.1 {
                log_error!(watcher.RemoveUpdated(token));
            }
            if let Some(token) = self.radio_removed_handler.1 {
                log_error!(watcher.RemoveRemoved(token));
            }
            log_error!(watcher.Stop());
        }

        for (radio, token) in self.radios.drain(..) {
            log_error!(radio.RemoveStateChanged(token));
        }
    }
}

fn on_radio_added(
    _sender: &Option<DeviceWatcher>,
    args: &Option<DeviceInformation>,
) -> windows_core::Result<()> {
    if let Some(device) = args {
        let id = device.Id()?.to_string_lossy();
        log_error!(RadioManager::event_tx().send(RadioManagerEvent::Added(id)));
    }
    Ok(())
}

fn on_radio_updated(
    _sender: &Option<DeviceWatcher>,
    args: &Option<DeviceInformationUpdate>,
) -> windows_core::Result<()> {
    if let Some(device) = args {
        let id = device.Id()?.to_string_lossy();
        log_error!(RadioManager::event_tx().send(RadioManagerEvent::Updated(id)));
    }
    Ok(())
}

fn on_radio_removed(
    _sender: &Option<DeviceWatcher>,
    args: &Option<DeviceInformationUpdate>,
) -> windows_core::Result<()> {
    if let Some(device) = args {
        let id = device.Id()?.to_string_lossy();
        log_error!(RadioManager::event_tx().send(RadioManagerEvent::Removed(id)));
    }
    Ok(())
}

fn on_radio_state_changed(
    _sender: &Option<Radio>,
    _args: &Option<windows_core::IInspectable>,
) -> windows_core::Result<()> {
    Ok(())
}
