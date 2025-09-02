mod device;
mod effects;
mod players;
mod session;

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use itertools::Itertools;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::system_state::MediaPlayerTimeline;
use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager, MediaPropertiesChangedEventArgs,
        PlaybackInfoChangedEventArgs, SessionsChangedEventArgs, TimelinePropertiesChangedEventArgs,
    },
    Win32::{
        Foundation::PROPERTYKEY,
        Media::Audio::{
            eAll, eCapture, eCommunications, eMultimedia, eRender, EDataFlow, ERole, IMMDevice,
            IMMDeviceEnumerator, IMMNotificationClient, IMMNotificationClient_Impl,
            MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::WinRT::EventRegistrationToken,
    },
};

use crate::{
    error::{ErrorMap, Result, ResultLogExt},
    event_manager, trace_lock,
    utils::pcwstr,
    windows_api::Com,
};

use super::domain::{MediaDevice, MediaDeviceSession, MediaDeviceType, MediaPlayer};

lazy_static! {
    pub static ref MEDIA_MANAGER: Arc<Mutex<MediaManager>> = Arc::new(Mutex::new(
        MediaManager::new().expect("Failed to create media manager")
    ));
}

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
    inputs: Vec<MediaDevice>,
    outputs: Vec<MediaDevice>,
    playing: HashMap<String, MediaPlayer>,

    device_enumerator: IMMDeviceEnumerator,
    mm_notification_client: IMMNotificationClient,

    media_player_manager: GlobalSystemMediaTransportControlsSessionManager,
    media_player_manager_event_handler: TypedEventHandler<
        GlobalSystemMediaTransportControlsSessionManager,
        SessionsChangedEventArgs,
    >,

    media_players: HashMap<String, GlobalSystemMediaTransportControlsSession>,
    media_player_properties_event_handler: TypedEventHandler<
        GlobalSystemMediaTransportControlsSession,
        MediaPropertiesChangedEventArgs,
    >,
    media_player_playback_event_handler:
        TypedEventHandler<GlobalSystemMediaTransportControlsSession, PlaybackInfoChangedEventArgs>,
    media_player_timeline_event_handler: TypedEventHandler<
        GlobalSystemMediaTransportControlsSession,
        TimelinePropertiesChangedEventArgs,
    >,
    /// session id -> (media properties changed event, playback info changed event)
    media_player_event_tokens: HashMap<
        String,
        (
            EventRegistrationToken,
            EventRegistrationToken,
            EventRegistrationToken,
        ),
    >,
}

unsafe impl Send for MediaManager {}

// getters/setters
impl MediaManager {
    pub fn inputs(&self) -> &Vec<MediaDevice> {
        &self.inputs
    }

    pub fn outputs(&self) -> &Vec<MediaDevice> {
        &self.outputs
    }

    pub fn playing(&self) -> Vec<&MediaPlayer> {
        self.playing.values().collect_vec()
    }

    pub fn device(&self, id: &str) -> Option<&MediaDevice> {
        self.inputs
            .iter()
            .chain(self.outputs.iter())
            .find(|d| d.id == id)
    }

    pub fn device_mut(&mut self, id: &str) -> Option<&mut MediaDevice> {
        self.inputs
            .iter_mut()
            .chain(self.outputs.iter_mut())
            .find(|d| d.id == id)
    }

    pub fn player_mut(&mut self, id: &str) -> Option<&mut MediaPlayer> {
        self.playing.get_mut(id)
    }

    pub fn get_raw_device(&self, device_id: &str) -> Option<IMMDevice> {
        unsafe { self.device_enumerator.GetDevice(pcwstr(device_id)) }.ok()
    }
}

impl MediaManager {
    pub fn new() -> Result<Self> {
        let media_player_manager =
            GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?;

        let manager = Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            playing: HashMap::new(),

            // unsafe com objects
            device_enumerator: Com::create_instance(&MMDeviceEnumerator)?,
            mm_notification_client: MediaManagerEvents.into(),

            media_player_manager,
            media_player_manager_event_handler: TypedEventHandler::new(
                MediaManagerEvents::on_media_players_changed,
            ),
            media_players: HashMap::new(),
            media_player_event_tokens: HashMap::new(),
            media_player_properties_event_handler: TypedEventHandler::new(
                MediaManagerEvents::on_media_player_properties_changed,
            ),
            media_player_timeline_event_handler: TypedEventHandler::new(
                MediaManagerEvents::on_media_player_timeline_changed,
            ),
            media_player_playback_event_handler: TypedEventHandler::new(
                MediaManagerEvents::on_media_player_playback_changed,
            ),
        };
        Ok(manager)
    }

    pub unsafe fn initialize(&mut self) -> Result<()> {
        let collection = self
            .device_enumerator
            .EnumAudioEndpoints(eAll, DEVICE_STATE_ACTIVE)?;

        for idx in 0..collection.GetCount()? {
            // log instead propagate error to avoid panic if just some device fail to load
            self.load_device(&collection.Item(idx)?).log_error();
        }

        self.device_enumerator
            .RegisterEndpointNotificationCallback(&self.mm_notification_client)?;

        for session in self.media_player_manager.GetSessions()? {
            self.load_media_transport_session(session)?;
        }

        self.update_recommended_player();
        self.media_player_manager
            .SessionsChanged(&self.media_player_manager_event_handler)?;

        Self::subscribe(|event| {
            let is_changing_players = matches!(
                event,
                MediaEvent::MediaPlayerAdded(_)
                    | MediaEvent::MediaPlayerRemoved(_)
                    | MediaEvent::MediaPlayerPropertiesChanged { .. }
                    | MediaEvent::MediaPlayerPlaybackStatusChanged { .. }
                    | MediaEvent::MediaPlayerTimelineChanged { .. }
            );

            let mut media_manager = trace_lock!(MEDIA_MANAGER);
            media_manager.process_event(event).log_error();

            if is_changing_players {
                media_manager.update_recommended_player();
            }
        });

        Ok(())
    }

    fn load_device(&mut self, device: &IMMDevice) -> Result<()> {
        let mut device = unsafe { MediaDevice::load(device)? };
        device.is_default_multimedia = self.is_default_device(&device, eMultimedia);
        device.is_default_communications = self.is_default_device(&device, eCommunications);
        match device.r#type {
            MediaDeviceType::Input => self.inputs.push(device),
            MediaDeviceType::Output => self.outputs.push(device),
        };
        Ok(())
    }

    fn remove_device(&mut self, device_id: &str) {
        for device in std::mem::take(&mut self.inputs) {
            if device.id == device_id {
                device.release();
                continue;
            }
            self.inputs.push(device);
        }
        for device in std::mem::take(&mut self.outputs) {
            if device.id == device_id {
                device.release();
                continue;
            }
            self.outputs.push(device);
        }
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

    fn process_event(&mut self, event: MediaEvent) -> Result<()> {
        match event {
            MediaEvent::DeviceAdded(device_id) => {
                if let Some(device) = self.get_raw_device(&device_id) {
                    self.load_device(&device)?;
                }
            }
            MediaEvent::DeviceRemoved(device_id) => {
                self.remove_device(&device_id);
            }
            MediaEvent::DefaultDeviceChanged {
                flow,
                role,
                device_id,
            } => {
                let devices = if flow == eCapture {
                    &mut self.inputs
                } else {
                    &mut self.outputs
                };

                for device in devices {
                    if role == eMultimedia {
                        device.is_default_multimedia = device.id == device_id;
                    } else if role == eCommunications {
                        device.is_default_communications = device.id == device_id;
                    }
                }
            }
            MediaEvent::DeviceVolumeChanged {
                device_id,
                volume,
                muted,
            } => {
                if let Some(device) = self.device_mut(&device_id) {
                    device.volume = volume;
                    device.muted = muted;
                }
            }
            MediaEvent::DeviceSessionAdded { device_id, session } => {
                if let Some(device) = self.device_mut(&device_id) {
                    device.sessions.push(session);
                }
            }
            MediaEvent::DeviceSessionRemoved {
                device_id,
                session_id,
            } => {
                if let Some(device) = self.device_mut(&device_id) {
                    device.remove_session(&session_id);
                }
            }
            MediaEvent::DeviceSessionVolumeChanged {
                device_id,
                session_id,
                volume,
                muted,
            } => {
                if let Some(device) = self.device_mut(&device_id) {
                    if let Some(session) = device.session_mut(&session_id) {
                        session.volume = volume;
                        session.muted = muted;
                    }
                }
            }
            MediaEvent::MediaPlayerAdded(session) => {
                if let Some(player) =
                    self.player_mut(&session.SourceAppUserModelId()?.to_string_lossy())
                {
                    player.removed_at = None;
                    return Ok(());
                }
                // load_media_transport_session could fail with 0x80070015 "The device is not ready."
                // when trying to load a recently added player so we retry a few times
                let mut max_attempts = 0;
                while session.TryGetMediaPropertiesAsync()?.get().is_err() && max_attempts < 15 {
                    max_attempts += 1;
                    std::thread::sleep(Duration::from_millis(10));
                }
                self.load_media_transport_session(session)?;
            }
            MediaEvent::MediaPlayerRemoved(id) => {
                if let Some(player) = self.player_mut(&id) {
                    player.removed_at = Some(std::time::Instant::now());
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(1500));
                        Self::send(MediaEvent::MediaPlayerCleanRequested);
                    });
                }
            }
            MediaEvent::MediaPlayerCleanRequested => {
                self.release_pending_players()?;
            }
            MediaEvent::MediaPlayerPropertiesChanged {
                id,
                title,
                author,
                thumbnail,
            } => {
                if let Some(player) = self.player_mut(&id) {
                    player.title = title;
                    player.author = author;
                    player.thumbnail = thumbnail;
                }
            }
            MediaEvent::MediaPlayerPlaybackStatusChanged { id, playing } => {
                if let Some(player) = self.player_mut(&id) {
                    player.playing = playing;
                }
            }
            MediaEvent::MediaPlayerTimelineChanged { id, timeline } => {
                if let Some(player) = self.player_mut(&id) {
                    player.timeline = timeline;
                }
            }
        }
        Ok(())
    }

    /// Release all resources
    /// should be called on application exit
    pub fn release(&mut self) {
        let keys = self.playing.keys().cloned().collect_vec();
        for id in keys {
            self.release_media_transport_session(&id).log_error();
        }

        for device in std::mem::take(&mut self.inputs) {
            device.release();
        }
        for device in std::mem::take(&mut self.outputs) {
            device.release();
        }
    }
}
