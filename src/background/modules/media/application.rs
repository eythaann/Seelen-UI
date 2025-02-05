use std::{collections::HashMap, ffi::OsStr, path::PathBuf, sync::Arc, time::Duration};

use itertools::Itertools;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus, MediaPropertiesChangedEventArgs,
        PlaybackInfoChangedEventArgs, SessionsChangedEventArgs,
    },
    Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Media::Audio::{
            eAll, eCapture, eCommunications, eConsole, eMultimedia, eRender, EDataFlow, ERole,
            Endpoints::{
                IAudioEndpointVolume, IAudioEndpointVolumeCallback,
                IAudioEndpointVolumeCallback_Impl,
            },
            IAudioSessionControl, IAudioSessionControl2, IAudioSessionEvents,
            IAudioSessionEvents_Impl, IAudioSessionManager2, IAudioSessionNotification,
            IAudioSessionNotification_Impl, IMMDevice, IMMDeviceEnumerator, IMMEndpoint,
            IMMNotificationClient, IMMNotificationClient_Impl, ISimpleAudioVolume,
            MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::Com::{CLSCTX_ALL, STGM_READ},
        UI::Shell::PropertiesSystem::PROPERTYKEY,
    },
};
use windows_core::Interface;

use crate::{
    error_handler::Result,
    event_manager, log_error,
    modules::start::application::START_MENU_MANAGER,
    seelen_weg::icon_extractor::extract_and_save_icon_umid,
    trace_lock,
    utils::pcwstr,
    windows_api::{Com, WindowsApi},
};

use super::domain::{
    Device, DeviceChannel, IPolicyConfig, MediaPlayer, MediaPlayerOwner, PolicyConfig,
};

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
    MediaPlayerAdded(GlobalSystemMediaTransportControlsSession),
    MediaPlayerRemoved(String),
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
}

#[windows_core::implement(IMMNotificationClient)]
struct MediaManagerEvents;

impl IMMNotificationClient_Impl for MediaManagerEvents_Impl {
    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        device_id: &windows_core::PCWSTR,
    ) -> windows_core::Result<()> {
        let tx = MediaManager::event_tx();
        let result = tx.send(MediaEvent::DefaultDeviceChanged {
            flow,
            role,
            device_id: unsafe { device_id.to_string()? },
        });
        log_error!(result);
        Ok(())
    }

    fn OnDeviceAdded(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        let tx = MediaManager::event_tx();
        let result = tx.send(MediaEvent::DeviceAdded(unsafe { device_id.to_string()? }));
        log_error!(result);
        Ok(())
    }

    fn OnDeviceRemoved(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        let tx = MediaManager::event_tx();
        let result = tx.send(MediaEvent::DeviceRemoved(unsafe { device_id.to_string()? }));
        log_error!(result);
        Ok(())
    }

    fn OnDeviceStateChanged(
        &self,
        device_id: &windows_core::PCWSTR,
        new_device_state: windows::Win32::Media::Audio::DEVICE_STATE,
    ) -> windows_core::Result<()> {
        let device_id = unsafe { device_id.to_string()? };
        let tx = MediaManager::event_tx();
        let result = match new_device_state {
            DEVICE_STATE_ACTIVE => tx.send(MediaEvent::DeviceAdded(device_id)),
            _ => tx.send(MediaEvent::DeviceRemoved(device_id)),
        };
        log_error!(result);
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

impl MediaManagerEvents {
    fn on_media_player_properties_changed(
        session: &Option<GlobalSystemMediaTransportControlsSession>,
        _args: &Option<MediaPropertiesChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session) = session {
            let id = session.SourceAppUserModelId()?.to_string();
            let properties = session.TryGetMediaPropertiesAsync()?.get()?;
            let tx = MediaManager::event_tx();
            let result = tx.send(MediaEvent::MediaPlayerPropertiesChanged {
                id,
                title: properties.Title()?.to_string(),
                author: properties.Artist()?.to_string(),
                thumbnail: WindowsApi::extract_thumbnail_from_ref(properties.Thumbnail()?).ok(),
            });
            log_error!(result);
        }
        Ok(())
    }

    fn on_media_player_playback_changed(
        session: &Option<GlobalSystemMediaTransportControlsSession>,
        _args: &Option<PlaybackInfoChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session) = session {
            let playback = session.GetPlaybackInfo()?;
            let tx = MediaManager::event_tx();
            let result = tx.send(MediaEvent::MediaPlayerPlaybackStatusChanged {
                id: session.SourceAppUserModelId()?.to_string(),
                playing: playback.PlaybackStatus()?
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
            });
            log_error!(result);
        }
        Ok(())
    }

    fn on_media_players_changed(
        session_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>,
        _args: &Option<SessionsChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session_manager) = session_manager {
            let mut current_list = {
                trace_lock!(MEDIA_MANAGER)
                    .playing()
                    .iter()
                    .map(|session| session.id.clone())
                    .collect_vec()
            };

            let tx = MediaManager::event_tx();
            for session in session_manager.GetSessions()? {
                let id = session.SourceAppUserModelId()?.to_string();
                if !current_list.contains(&id) {
                    let _ = tx.send(MediaEvent::MediaPlayerAdded(session));
                }
                current_list.retain(|x| *x != id);
            }

            for id in current_list {
                let _ = tx.send(MediaEvent::MediaPlayerRemoved(id));
            }
        }
        Ok(())
    }
}

#[windows::core::implement(IAudioEndpointVolumeCallback, IAudioSessionNotification)]
struct MediaDeviceEventHandler {
    device_id: String,
}

impl IAudioEndpointVolumeCallback_Impl for MediaDeviceEventHandler_Impl {
    fn OnNotify(
        &self,
        data: *mut windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA,
    ) -> windows_core::Result<()> {
        if let Some(data) = unsafe { data.as_ref() } {
            let tx = MediaManager::event_tx();
            let result = tx.send(MediaEvent::DeviceVolumeChanged {
                device_id: self.device_id.clone(),
                volume: data.fMasterVolume,
                muted: data.bMuted.as_bool(),
            });
            log_error!(result);
        }
        Ok(())
    }
}

impl IAudioSessionNotification_Impl for MediaDeviceEventHandler_Impl {
    fn OnSessionCreated(
        &self,
        _new_session: Option<&IAudioSessionControl>,
    ) -> windows::core::Result<()> {
        // println!("SESSION CREATED!")
        Ok(())
    }
}

#[windows::core::implement(IAudioSessionEvents)]
struct MediaSessionEventHandler;

impl IAudioSessionEvents_Impl for MediaSessionEventHandler_Impl {
    fn OnChannelVolumeChanged(
        &self,
        _channel_count: u32,
        _new_channel_volume_array: *const f32,
        _changed_channel: u32,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        // println!("CHANNEL VOLUME CHANGED!");
        Ok(())
    }

    fn OnDisplayNameChanged(
        &self,
        _new_display_name: &windows::core::PCWSTR,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        // println!("DISPLAY NAME CHANGED!");
        Ok(())
    }

    fn OnGroupingParamChanged(
        &self,
        _new_grouping_param: *const windows::core::GUID,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        // println!("GROUPING PARAM CHANGED!");
        Ok(())
    }

    fn OnIconPathChanged(
        &self,
        _new_icon_path: &windows::core::PCWSTR,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        // println!("ICON PATH CHANGED!");
        Ok(())
    }

    fn OnSessionDisconnected(
        &self,
        _disconnect_reason: windows::Win32::Media::Audio::AudioSessionDisconnectReason,
    ) -> windows::core::Result<()> {
        // println!("SESSION DISCONNECTED!");
        Ok(())
    }

    fn OnSimpleVolumeChanged(
        &self,
        _new_volume: f32,
        _new_mute: windows::Win32::Foundation::BOOL,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        // println!("SIMPLE VOLUME CHANGED!");
        Ok(())
    }

    fn OnStateChanged(
        &self,
        _new_state: windows::Win32::Media::Audio::AudioSessionState,
    ) -> windows::core::Result<()> {
        // println!("STATE CHANGED! {:?}", _new_state);
        Ok(())
    }
}

type OnDevicesChange = Box<dyn Fn(&Vec<Device>, &Vec<Device>) + Send + Sync>;
type OnPlayersChange = Box<dyn Fn(&Vec<MediaPlayer>) + Send + Sync>;
pub struct MediaManager {
    inputs: Vec<Device>,
    outputs: Vec<Device>,
    playing: Vec<MediaPlayer>,

    registered_devices_callbacks: Vec<OnDevicesChange>,
    registered_players_callbacks: Vec<OnPlayersChange>,

    device_enumerator: IMMDeviceEnumerator,
    mm_notification_client: IMMNotificationClient,
    devices_audio_endpoint: HashMap<String, (IAudioEndpointVolume, IAudioEndpointVolumeCallback)>,

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
    /// session id -> (media properties changed event, playback info changed event)
    media_player_event_tokens: HashMap<String, (EventRegistrationToken, EventRegistrationToken)>,
}

unsafe impl Send for MediaManager {}

// getters/setters
impl MediaManager {
    pub fn inputs(&self) -> &Vec<Device> {
        &self.inputs
    }

    pub fn outputs(&self) -> &Vec<Device> {
        &self.outputs
    }

    pub fn playing(&self) -> &Vec<MediaPlayer> {
        &self.playing
    }

    pub fn device_mut(&mut self, id: &str) -> Option<&mut Device> {
        self.inputs
            .iter_mut()
            .chain(self.outputs.iter_mut())
            .find(|d| d.id == id)
    }

    pub fn player_mut(&mut self, id: &str) -> Option<&mut MediaPlayer> {
        self.playing.iter_mut().find(|p| p.id == id)
    }

    pub fn get_raw_device(&self, device_id: &str) -> Option<IMMDevice> {
        unsafe { self.device_enumerator.GetDevice(pcwstr(device_id)) }.ok()
    }

    pub fn devices_audio_endpoint(
        &self,
    ) -> &HashMap<String, (IAudioEndpointVolume, IAudioEndpointVolumeCallback)> {
        &self.devices_audio_endpoint
    }

    pub fn session_by_id(&self, id: &str) -> Option<&GlobalSystemMediaTransportControlsSession> {
        self.media_players.get(id)
    }

    pub fn get_recommended_player_id(&self) -> Result<String> {
        Ok(self
            .media_player_manager
            .GetCurrentSession()?
            .SourceAppUserModelId()?
            .to_string_lossy())
    }
}

impl MediaManager {
    pub fn new() -> Result<Self> {
        let media_player_manager =
            GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?;

        let mut manager = Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            playing: Vec::new(),
            registered_devices_callbacks: Vec::new(),
            registered_players_callbacks: Vec::new(),

            // unsafe com objects
            devices_audio_endpoint: HashMap::new(),
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
            media_player_playback_event_handler: TypedEventHandler::new(
                MediaManagerEvents::on_media_player_playback_changed,
            ),
        };

        unsafe { manager.initialize()? };
        Ok(manager)
    }

    pub fn on_change_players<F>(&mut self, callback: F)
    where
        F: Fn(&Vec<MediaPlayer>) + Send + Sync + 'static,
    {
        self.registered_players_callbacks.push(Box::new(callback));
    }

    pub fn on_change_devices<F>(&mut self, callback: F)
    where
        F: Fn(&Vec<Device>, &Vec<Device>) + Send + Sync + 'static,
    {
        self.registered_devices_callbacks.push(Box::new(callback));
    }

    unsafe fn initialize(&mut self) -> Result<()> {
        let collection = self
            .device_enumerator
            .EnumAudioEndpoints(eAll, DEVICE_STATE_ACTIVE)?;

        for idx in 0..collection.GetCount()? {
            self.load_device(&collection.Item(idx)?)?;
        }

        self.device_enumerator
            .RegisterEndpointNotificationCallback(&self.mm_notification_client)?;

        for session in self.media_player_manager.GetSessions()? {
            self.load_media_transport_session(session)?;
        }

        self.update_recommended_player();
        self.media_player_manager
            .SessionsChanged(&self.media_player_manager_event_handler)?;
        Self::start_event_loop();
        Ok(())
    }

    unsafe fn load_device(&mut self, device: &IMMDevice) -> Result<()> {
        let device_id = device.GetId()?.to_string()?;
        let device_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
        let device_session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;

        // create Serializable Device object
        let mut sessions = Vec::new();
        let enumerator = device_session_manager.GetSessionEnumerator()?;
        for session_idx in 0..enumerator.GetCount()? {
            let session: IAudioSessionControl2 = enumerator.GetSession(session_idx)?.cast()?;
            let volume: ISimpleAudioVolume = session.cast()?;
            // let process = Process::from_id(session.GetProcessId()?);
            let name = String::new(); // todo on media mixer feature
            let icon_path = None; // todo on media mixer feature

            sessions.push(DeviceChannel {
                id: session.GetSessionIdentifier()?.to_string()?,
                instance_id: session.GetSessionInstanceIdentifier()?.to_string()?,
                process_id: session.GetProcessId()?,
                name,
                icon_path,
                is_system: session.IsSystemSoundsSession().0 == 0,
                volume: volume.GetMasterVolume()?,
                muted: volume.GetMute()?.as_bool(),
            });
        }

        let is_input = device.cast::<IMMEndpoint>()?.GetDataFlow()? == eCapture;
        let properties = device.OpenPropertyStore(STGM_READ)?;

        let (is_default_multimedia, is_default_communications) = if is_input {
            (
                self.is_default_device(&device_id, eCapture, eMultimedia),
                self.is_default_device(&device_id, eCapture, eCommunications),
            )
        } else {
            (
                self.is_default_device(&device_id, eRender, eMultimedia),
                self.is_default_device(&device_id, eRender, eCommunications),
            )
        };

        let device = Device {
            id: device_id.clone(),
            name: properties.GetValue(&PKEY_Device_FriendlyName)?.to_string(),
            is_default_multimedia,
            is_default_communications,
            sessions,
            volume: device_volume.GetMasterVolumeLevelScalar()?,
            muted: device_volume.GetMute()?.as_bool(),
        };

        if is_input {
            self.inputs.push(device);
        } else {
            self.outputs.push(device);
        }

        // listen for device events
        let device_volume_callback = IAudioEndpointVolumeCallback::from(MediaDeviceEventHandler {
            device_id: device_id.clone(),
        });

        device_volume.RegisterControlChangeNotify(&device_volume_callback)?;
        self.devices_audio_endpoint
            .insert(device_id, (device_volume, device_volume_callback));
        Ok(())
    }

    fn release_device(&mut self, device_id: &str) -> Result<()> {
        if let Some((endpoint, callback)) = self.devices_audio_endpoint.remove(device_id) {
            unsafe { endpoint.UnregisterControlChangeNotify(&callback)? };
            // avoid call drop and IUnknown::Release because the COM object was removed on device disconnection
            // and call Release on a unexciting object can produce a deadlock
            std::mem::forget(endpoint);
            std::mem::forget(callback);
        }
        self.inputs.retain(|d| d.id != device_id);
        self.outputs.retain(|d| d.id != device_id);
        Ok(())
    }

    fn load_media_transport_session(
        &mut self,
        session: GlobalSystemMediaTransportControlsSession,
    ) -> Result<()> {
        let source_app_umid = session.SourceAppUserModelId()?.to_string_lossy();
        let properties = session.TryGetMediaPropertiesAsync()?.get()?;

        let playback_info = session.GetPlaybackInfo()?;
        let status = playback_info.PlaybackStatus()?;

        let display_name = if WindowsApi::is_uwp_package_id(&source_app_umid) {
            WindowsApi::get_uwp_app_info(&source_app_umid)?
                .DisplayInfo()?
                .DisplayName()?
                .to_string_lossy()
        } else {
            let shortcut = START_MENU_MANAGER
                .load()
                .search_shortcut_with_same_umid(&source_app_umid);
            match shortcut {
                Some(shortcut) => shortcut
                    .file_stem()
                    .unwrap_or_else(|| OsStr::new("Unknown"))
                    .to_string_lossy()
                    .to_string(),
                None => "Unknown".to_string(),
            }
        };

        self.playing.push(MediaPlayer {
            id: source_app_umid.clone(),
            title: properties.Title().unwrap_or_default().to_string_lossy(),
            author: properties.Artist().unwrap_or_default().to_string_lossy(),
            owner: Some(MediaPlayerOwner {
                name: display_name,
                icon_path: extract_and_save_icon_umid(&source_app_umid).ok(),
            }),
            thumbnail: properties
                .Thumbnail()
                .ok()
                .and_then(|stream| WindowsApi::extract_thumbnail_from_ref(stream).ok()),
            playing: status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
            default: false,
        });

        // listen for media transport events
        self.media_player_event_tokens.insert(
            source_app_umid.clone(),
            (
                session.MediaPropertiesChanged(&self.media_player_properties_event_handler)?,
                session.PlaybackInfoChanged(&self.media_player_playback_event_handler)?,
            ),
        );
        self.media_players.insert(source_app_umid, session);
        Ok(())
    }

    fn update_recommended_player(&mut self) {
        if let Ok(recommended) = self.get_recommended_player_id() {
            for player in &mut self.playing {
                player.default = player.id == recommended;
            }
        }
    }

    fn release_media_transport_session(&mut self, player_id: &str) -> Result<()> {
        if let Some(session) = self.media_players.remove(player_id) {
            if let Some((properties_token, playback_token)) =
                self.media_player_event_tokens.remove(player_id)
            {
                session.RemoveMediaPropertiesChanged(properties_token)?;
                session.RemovePlaybackInfoChanged(playback_token)?;
            }
        }
        self.playing.retain(|player| player.id != player_id);
        Ok(())
    }

    fn is_default_device(&self, device_id: &str, dataflow: EDataFlow, role: ERole) -> bool {
        unsafe {
            self.device_enumerator
                .GetDefaultAudioEndpoint(dataflow, role)
                .and_then(|d| d.GetId())
                .and_then(|id| id.to_hstring())
                .map(|id| id.to_string())
                .map(|id| id == device_id)
                .unwrap_or(false)
        }
    }

    fn start_event_loop() {
        Self::subscribe(|event| {
            let is_changing_players = matches!(
                event,
                MediaEvent::MediaPlayerAdded(_)
                    | MediaEvent::MediaPlayerRemoved(_)
                    | MediaEvent::MediaPlayerPropertiesChanged { .. }
                    | MediaEvent::MediaPlayerPlaybackStatusChanged { .. }
            );

            let mut media_manager = trace_lock!(MEDIA_MANAGER);
            log_error!(media_manager.process_event(event));

            if is_changing_players {
                media_manager.update_recommended_player();
                for callback in &media_manager.registered_players_callbacks {
                    callback(media_manager.playing());
                }
            } else {
                for callback in &media_manager.registered_devices_callbacks {
                    callback(media_manager.inputs(), media_manager.outputs());
                }
            }
        });
    }

    fn process_event(&mut self, event: MediaEvent) -> Result<()> {
        match event {
            MediaEvent::DeviceAdded(device_id) => {
                if let Some(device) = self.get_raw_device(&device_id) {
                    unsafe { self.load_device(&device)? };
                }
            }
            MediaEvent::DeviceRemoved(device_id) => {
                self.release_device(&device_id)?;
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
            MediaEvent::MediaPlayerAdded(session) => {
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
                self.release_media_transport_session(&id)?;
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
        }
        Ok(())
    }

    pub fn set_default_device(&mut self, id: &str, role: &str) -> Result<()> {
        let role = match role {
            "multimedia" => eMultimedia,
            "communications" => eCommunications,
            "console" => eConsole,
            _ => return Err("invalid role".into()),
        };

        let policy: IPolicyConfig = Com::create_instance(&PolicyConfig)?;
        unsafe {
            policy.SetDefaultEndpoint(pcwstr(id), role)?;
        }
        Ok(())
    }

    /// Release all resources
    /// should be called on application exit
    pub fn release(&mut self) {
        let player_ids = self.playing.iter().map(|p| p.id.clone()).collect_vec();

        for player_id in player_ids {
            log_error!(self.release_media_transport_session(&player_id));
        }

        let device_ids = self
            .inputs
            .iter()
            .map(|d| d.id.clone())
            .chain(self.outputs.iter().map(|d| d.id.clone()))
            .collect_vec();

        for device_id in device_ids {
            log_error!(self.release_device(&device_id));
        }
    }
}
