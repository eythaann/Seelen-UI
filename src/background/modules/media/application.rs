use std::{collections::HashMap, path::PathBuf, sync::Arc};

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
        Storage::EnhancedStorage::PKEY_FileDescription,
        System::Com::{CLSCTX_ALL, STGM_READ},
        UI::Shell::{PropertiesSystem::PROPERTYKEY, SIGDN_NORMALDISPLAY},
    },
};
use windows_core::Interface;

use crate::{
    error_handler::Result,
    log_error,
    seelen::get_app_handle,
    seelen_weg::icon_extractor::{extract_and_save_icon, extract_and_save_icon_v2},
    trace_lock,
    utils::pcwstr,
    windows_api::{Com, WindowEnumerator, WindowsApi},
};

use super::domain::{
    Device, DeviceChannel, IPolicyConfig, MediaPlayer, MediaPlayerOwner, PolicyConfig,
};

lazy_static! {
    pub static ref MEDIA_MANAGER: Arc<Mutex<MediaManager>> = Arc::new(Mutex::new(
        MediaManager::new().expect("Failed to create media manager")
    ));
    pub static ref REG_PROPERTY_EVENTS: Arc<Mutex<HashMap<String, EventRegistrationToken>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref REG_PLAYBACK_EVENTS: Arc<Mutex<HashMap<String, EventRegistrationToken>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug)]
enum MediaEvent {
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

impl IMMNotificationClient_Impl for MediaManagerEvents {
    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        device_id: &windows_core::PCWSTR,
    ) -> windows_core::Result<()> {
        trace_lock!(MEDIA_MANAGER).emit_event(MediaEvent::DefaultDeviceChanged {
            flow,
            role,
            device_id: unsafe { device_id.to_string()? },
        });
        Ok(())
    }

    fn OnDeviceAdded(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        trace_lock!(MEDIA_MANAGER)
            .emit_event(MediaEvent::DeviceAdded(unsafe { device_id.to_string()? }));
        Ok(())
    }

    fn OnDeviceRemoved(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        trace_lock!(MEDIA_MANAGER)
            .emit_event(MediaEvent::DeviceRemoved(unsafe { device_id.to_string()? }));
        Ok(())
    }

    fn OnDeviceStateChanged(
        &self,
        device_id: &windows_core::PCWSTR,
        new_device_state: windows::Win32::Media::Audio::DEVICE_STATE,
    ) -> windows_core::Result<()> {
        let device_id = unsafe { device_id.to_string()? };
        match new_device_state {
            DEVICE_STATE_ACTIVE => {
                trace_lock!(MEDIA_MANAGER).emit_event(MediaEvent::DeviceAdded(device_id));
            }
            _ => {
                trace_lock!(MEDIA_MANAGER).emit_event(MediaEvent::DeviceRemoved(device_id));
            }
        }
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
            trace_lock!(MEDIA_MANAGER).emit_event(MediaEvent::MediaPlayerPropertiesChanged {
                id,
                title: properties.Title()?.to_string(),
                author: properties.Artist()?.to_string(),
                thumbnail: WindowsApi::extract_thumbnail_from_ref(properties.Thumbnail()?).ok(),
            });
        }
        Ok(())
    }

    fn on_media_player_playback_changed(
        session: &Option<GlobalSystemMediaTransportControlsSession>,
        _args: &Option<PlaybackInfoChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session) = session {
            let playback = session.GetPlaybackInfo()?;
            trace_lock!(MEDIA_MANAGER).emit_event(MediaEvent::MediaPlayerPlaybackStatusChanged {
                id: session.SourceAppUserModelId()?.to_string(),
                playing: playback.PlaybackStatus()?
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
            });
        }
        Ok(())
    }

    fn on_media_players_changed(
        session_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>,
        _args: &Option<SessionsChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session_manager) = session_manager {
            let mut current_list = trace_lock!(MEDIA_MANAGER)
                .playing()
                .iter()
                .map(|session| session.id.clone())
                .collect_vec();

            for session in session_manager.GetSessions()? {
                let id = session.SourceAppUserModelId()?.to_string();
                if !current_list.contains(&id) {
                    trace_lock!(MEDIA_MANAGER).emit_event(MediaEvent::MediaPlayerAdded(session));
                }
                current_list.retain(|x| *x != id);
            }

            for id in current_list {
                trace_lock!(MEDIA_MANAGER).emit_event(MediaEvent::MediaPlayerRemoved(id));
            }
        }
        Ok(())
    }
}

#[windows::core::implement(IAudioEndpointVolumeCallback, IAudioSessionNotification)]
struct MediaDeviceEventHandler {
    device_id: String,
}

impl IAudioEndpointVolumeCallback_Impl for MediaDeviceEventHandler {
    fn OnNotify(
        &self,
        data: *mut windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA,
    ) -> windows_core::Result<()> {
        if let Some(data) = unsafe { data.as_ref() } {
            trace_lock!(MEDIA_MANAGER).emit_event(MediaEvent::DeviceVolumeChanged {
                device_id: self.device_id.clone(),
                volume: data.fMasterVolume,
                muted: data.bMuted.as_bool(),
            });
        }
        Ok(())
    }
}

impl IAudioSessionNotification_Impl for MediaDeviceEventHandler {
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

impl IAudioSessionEvents_Impl for MediaSessionEventHandler {
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

            let name;
            let icon_path;
            match WindowsApi::exe_path_by_process(session.GetProcessId()?) {
                Ok(path) => {
                    let shell_item = WindowsApi::get_shell_item(&path)?;
                    name = match shell_item.GetString(&PKEY_FileDescription) {
                        Ok(description) => description.to_string()?,
                        Err(_) => shell_item
                            .GetDisplayName(SIGDN_NORMALDISPLAY)?
                            .to_string()?,
                    }
                    .replace(".exe", "");
                    icon_path = extract_and_save_icon(&get_app_handle(), &path)
                        .ok()
                        .map(|p| p.to_string_lossy().to_string());
                }
                Err(_) => {
                    name = session.GetDisplayName()?.to_string()?;
                    icon_path = None;
                }
            }

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
        let source_app_user_model_id = session.SourceAppUserModelId()?.to_string_lossy();
        let properties = session.TryGetMediaPropertiesAsync()?.get()?;

        let playback_info = session.GetPlaybackInfo()?;
        let status = playback_info.PlaybackStatus()?;

        let owner = WindowEnumerator::new().find(|w| {
            if let Some(id) = w.app_user_model_id() {
                return id == source_app_user_model_id;
            }
            false
        })?;

        self.playing.push(MediaPlayer {
            id: source_app_user_model_id.clone(),
            title: properties.Title().unwrap_or_default().to_string_lossy(),
            author: properties.Artist().unwrap_or_default().to_string_lossy(),
            owner: owner.map(|w| MediaPlayerOwner {
                name: w.title(),
                icon_path: w.exe().and_then(extract_and_save_icon_v2).ok(),
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
            source_app_user_model_id.clone(),
            (
                session.MediaPropertiesChanged(&self.media_player_properties_event_handler)?,
                session.PlaybackInfoChanged(&self.media_player_playback_event_handler)?,
            ),
        );
        self.media_players.insert(source_app_user_model_id, session);
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

    fn emit_event(&mut self, event: MediaEvent) {
        let is_changing_players = matches!(
            event,
            MediaEvent::MediaPlayerAdded(_)
                | MediaEvent::MediaPlayerRemoved(_)
                | MediaEvent::MediaPlayerPropertiesChanged { .. }
                | MediaEvent::MediaPlayerPlaybackStatusChanged { .. }
        );

        log_error!(self.process_event(event));

        if is_changing_players {
            self.update_recommended_player();
            for callback in &self.registered_players_callbacks {
                callback(self.playing());
            }
        } else {
            for callback in &self.registered_devices_callbacks {
                callback(self.inputs(), self.outputs());
            }
        }
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
