mod device;
mod effects;
mod players;
mod session;

use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use seelen_core::system_state::MediaPlayerTimeline;
use windows::{
    Media::Control::GlobalSystemMediaTransportControlsSession,
    Win32::{
        Foundation::PROPERTYKEY,
        Media::Audio::{
            eAll, eCapture, eCommunications, eMultimedia, eRender, EDataFlow, ERole, IMMDevice,
            IMMDeviceEnumerator, IMMNotificationClient, IMMNotificationClient_Impl,
            MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
    },
};

use crate::{
    error::{ErrorMap, Result, ResultLogExt},
    event_manager,
    utils::{lock_free::SyncHashMap, pcwstr},
    windows_api::Com,
};

use super::domain::{MediaDevice, MediaDeviceSession, MediaDeviceType};

pub static MEDIA_MANAGER: LazyLock<Arc<MediaManager>> =
    LazyLock::new(|| Arc::new(MediaManager::new().expect("Failed to create media manager")));

event_manager!(MediaManager, MediaEvent);

#[derive(Debug, Clone)]
pub enum MediaEvent {
    DeviceAdded(String),
    DeviceRemoved(String),
    DefaultDeviceChanged {
        flow: EDataFlow,
        role: ERole,
        device_id: String,
    },
    DeviceVolumeChanged {
        device_id: String,
        volume: f32,
        muted: bool,
    },
    // == sessions ==
    DeviceSessionAdded {
        device_id: String,
        session: MediaDeviceSession,
    },
    DeviceSessionRemoved {
        device_id: String,
        session_id: String,
    },
    DeviceSessionVolumeChanged {
        device_id: String,
        session_id: String,
        volume: f32,
        muted: bool,
    },
    // == players ==
    MediaPlayerAdded(GlobalSystemMediaTransportControlsSession),
    MediaPlayerRemoved(String),
    MediaPlayerCleanRequested,
    MediaPlayerPropertiesChanged {
        id: String,
        title: String,
        author: String,
        thumbnail: Option<PathBuf>,
    },
    MediaPlayerPlaybackStatusChanged {
        id: String,
        playing: bool,
    },
    MediaPlayerTimelineChanged {
        id: String,
        timeline: MediaPlayerTimeline,
    },
}

unsafe impl Send for MediaEvent {}

#[windows_core::implement(IMMNotificationClient)]
struct MediaManagerEvents;

impl IMMNotificationClient_Impl for MediaManagerEvents_Impl {
    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        device_id: &windows_core::PCWSTR,
    ) -> windows_core::Result<()> {
        MediaManager::send(MediaEvent::DefaultDeviceChanged {
            flow,
            role,
            device_id: unsafe { device_id.to_string()? },
        });
        Ok(())
    }

    fn OnDeviceAdded(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        MediaManager::send(MediaEvent::DeviceAdded(unsafe { device_id.to_string()? }));
        Ok(())
    }

    fn OnDeviceRemoved(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        MediaManager::send(MediaEvent::DeviceRemoved(unsafe { device_id.to_string()? }));
        Ok(())
    }

    fn OnDeviceStateChanged(
        &self,
        device_id: &windows_core::PCWSTR,
        new_device_state: windows::Win32::Media::Audio::DEVICE_STATE,
    ) -> windows_core::Result<()> {
        let device_id = unsafe { device_id.to_string()? };
        let tx = MediaManager::event_tx();
        match new_device_state {
            DEVICE_STATE_ACTIVE => tx.send(MediaEvent::DeviceAdded(device_id)),
            _ => tx.send(MediaEvent::DeviceRemoved(device_id)),
        }
        .wrap_error()
        .log_error();
        Ok(())
    }

    fn OnPropertyValueChanged(
        &self,
        _device_id: &windows_core::PCWSTR,
        _key: &PROPERTYKEY,
    ) -> windows_core::Result<()> {
        Ok(())
    }
}

pub struct MediaManager {
    pub inputs: SyncHashMap<String, MediaDevice>,
    pub outputs: SyncHashMap<String, MediaDevice>,

    device_enumerator: IMMDeviceEnumerator,
    mm_notification_client: IMMNotificationClient,

    pub players: players::MediaPlayersManager,
}

unsafe impl Send for MediaManager {}
unsafe impl Sync for MediaManager {}

// getters/setters
impl MediaManager {
    pub fn get_raw_device(&self, device_id: &str) -> Option<IMMDevice> {
        unsafe { self.device_enumerator.GetDevice(pcwstr(device_id)) }.ok()
    }

    // Media players getters
    pub fn get_media_player(
        &self,
        umid: &str,
    ) -> Option<GlobalSystemMediaTransportControlsSession> {
        self.players.get_media_player(umid)
    }
}

impl MediaManager {
    pub fn new() -> Result<Self> {
        let manager = Self {
            inputs: SyncHashMap::new(),
            outputs: SyncHashMap::new(),

            // unsafe com objects
            device_enumerator: Com::create_instance(&MMDeviceEnumerator)?,
            mm_notification_client: MediaManagerEvents.into(),

            players: players::MediaPlayersManager::new()?,
        };
        Ok(manager)
    }

    pub unsafe fn initialize(&self) -> Result<()> {
        let collection = self
            .device_enumerator
            .EnumAudioEndpoints(eAll, DEVICE_STATE_ACTIVE)?;

        for idx in 0..collection.GetCount()? {
            // log instead propagate error to avoid panic if just some device fail to load
            self.load_device(&collection.Item(idx)?).log_error();
        }

        self.device_enumerator
            .RegisterEndpointNotificationCallback(&self.mm_notification_client)?;

        self.players.initialize()?;

        Self::subscribe(|event| {
            let is_changing_players = matches!(
                event,
                MediaEvent::MediaPlayerAdded(_)
                    | MediaEvent::MediaPlayerRemoved(_)
                    | MediaEvent::MediaPlayerPropertiesChanged { .. }
                    | MediaEvent::MediaPlayerPlaybackStatusChanged { .. }
                    | MediaEvent::MediaPlayerTimelineChanged { .. }
            );

            MEDIA_MANAGER.process_event(event).log_error();

            if is_changing_players {
                MEDIA_MANAGER.players.update_recommended_player();
            }
        });

        Ok(())
    }

    fn load_device(&self, device: &IMMDevice) -> Result<()> {
        let mut device = unsafe { MediaDevice::load(device)? };
        device.is_default_multimedia = self.is_default_device(&device, eMultimedia);
        device.is_default_communications = self.is_default_device(&device, eCommunications);
        match device.r#type {
            MediaDeviceType::Input => self.inputs.upsert(device.id.clone(), device),
            MediaDeviceType::Output => self.outputs.upsert(device.id.clone(), device),
        };
        Ok(())
    }

    fn remove_device(&self, device_id: &str) {
        if let Some(device) = self.inputs.remove(device_id) {
            device.release();
        };
        if let Some(device) = self.outputs.remove(device_id) {
            device.release();
        };
    }

    fn is_default_device(&self, device: &MediaDevice, role: ERole) -> bool {
        let dataflow = match device.r#type {
            MediaDeviceType::Input => eCapture,
            MediaDeviceType::Output => eRender,
        };
        unsafe {
            self.device_enumerator
                .GetDefaultAudioEndpoint(dataflow, role)
                .and_then(|d| d.GetId())
                .map(|id| id.to_hstring() == device.id)
                .unwrap_or(false)
        }
    }

    fn process_event(&self, event: MediaEvent) -> Result<()> {
        match &event {
            MediaEvent::DeviceAdded(device_id) => {
                if let Some(device) = self.get_raw_device(device_id) {
                    self.load_device(&device)?;
                }
            }
            MediaEvent::DeviceRemoved(device_id) => {
                self.remove_device(device_id);
            }
            MediaEvent::DefaultDeviceChanged {
                flow,
                role,
                device_id,
            } => {
                let devices = if *flow == eCapture {
                    &self.inputs
                } else {
                    &self.outputs
                };

                devices.for_each(|(_, device)| {
                    if *role == eMultimedia {
                        device.is_default_multimedia = device.id == *device_id;
                    } else if *role == eCommunications {
                        device.is_default_communications = device.id == *device_id;
                    }
                });
            }
            MediaEvent::DeviceVolumeChanged {
                device_id,
                volume,
                muted,
            } => {
                let cb = |device: &mut MediaDevice| {
                    device.volume = *volume;
                    device.muted = *muted;
                };
                self.inputs.get(device_id, cb);
                self.outputs.get(device_id, cb);
            }
            MediaEvent::DeviceSessionAdded { device_id, session } => {
                let cb = |device: &mut MediaDevice| {
                    device.sessions.push(session.clone());
                };
                self.inputs.get(device_id, cb);
                self.outputs.get(device_id, cb);
            }
            MediaEvent::DeviceSessionRemoved {
                device_id,
                session_id,
            } => {
                let cb = |device: &mut MediaDevice| {
                    device.remove_session(session_id);
                };
                self.inputs.get(device_id, cb);
                self.outputs.get(device_id, cb);
            }
            MediaEvent::DeviceSessionVolumeChanged {
                device_id,
                session_id,
                volume,
                muted,
            } => {
                let cb = |device: &mut MediaDevice| {
                    if let Some(session) = device.session_mut(session_id) {
                        session.volume = *volume;
                        session.muted = *muted;
                    }
                };
                self.inputs.get(device_id, cb);
                self.outputs.get(device_id, cb);
            }
            MediaEvent::MediaPlayerAdded(_)
            | MediaEvent::MediaPlayerRemoved(_)
            | MediaEvent::MediaPlayerCleanRequested
            | MediaEvent::MediaPlayerPropertiesChanged { .. }
            | MediaEvent::MediaPlayerPlaybackStatusChanged { .. }
            | MediaEvent::MediaPlayerTimelineChanged { .. } => {
                self.players.process_event(&event)?;
            }
        }
        Ok(())
    }

    /// Release all resources
    /// should be called on application exit
    pub fn release(&self) {
        self.players.release();
        self.inputs.clear();
        self.outputs.clear();
    }
}
