use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    },
    Storage::Streams::{Buffer, DataReader, InputStreamOptions},
    Win32::{
        Media::Audio::{
            eMultimedia, eRender,
            Endpoints::{
                IAudioEndpointVolume, IAudioEndpointVolumeCallback,
                IAudioEndpointVolumeCallback_Impl,
            },
            IAudioSessionControl, IAudioSessionEvents, IAudioSessionEvents_Impl,
            IAudioSessionManager2, IAudioSessionNotification, IAudioSessionNotification_Impl,
            IMMDeviceEnumerator, MMDeviceEnumerator,
        },
        System::Com::CLSCTX_ALL,
    },
};

use crate::{error_handler::Result, log_error, windows_api::Com};

use super::domain::MediaSession;

lazy_static! {
    pub static ref MEDIA_MANAGER: Arc<Mutex<MediaManager>> =
        Arc::new(Mutex::new(MediaManager::new()));
    pub static ref REG_PROPERTY_EVENTS: Arc<Mutex<HashMap<String, EventRegistrationToken>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref REG_PLAYBACK_EVENTS: Arc<Mutex<HashMap<String, EventRegistrationToken>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[windows::core::implement(IAudioEndpointVolumeCallback)]
pub struct AudioEndpointVolumeCallback<F>
where
    F: Fn(f32, bool) + Send + 'static,
{
    callback: F,
}

impl<F> IAudioEndpointVolumeCallback_Impl for AudioEndpointVolumeCallback<F>
where
    F: Fn(f32, bool) + Send + 'static,
{
    fn OnNotify(
        &self,
        data: *mut windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA,
    ) -> windows_core::Result<()> {
        if data.is_null() {
            return Ok(());
        }
        let data = unsafe { *data };
        (self.callback)(data.fMasterVolume, data.bMuted.as_bool());
        Ok(())
    }
}

#[windows::core::implement(IAudioSessionEvents)]
pub struct MediaSessionEventHandler;

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
        println!("STATE CHANGED! {:?}", _new_state);
        Ok(())
    }
}

#[windows::core::implement(IAudioSessionNotification)]
pub struct MediaSessionCreationEventHandler {
    event_handler: IAudioSessionEvents,
    sessions: Mutex<Vec<IAudioSessionControl>>,
}

impl Default for MediaSessionCreationEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaSessionCreationEventHandler {
    pub fn new() -> Self {
        Self {
            event_handler: MediaSessionEventHandler.into(),
            sessions: Mutex::new(Vec::new()),
        }
    }
}

impl IAudioSessionNotification_Impl for MediaSessionCreationEventHandler {
    fn OnSessionCreated(
        &self,
        new_session: Option<&IAudioSessionControl>,
    ) -> windows::core::Result<()> {
        if let Some(session) = new_session {
            println!("SESSION CREATED! {}", unsafe {
                session.GetDisplayName()?.to_string()?
            });
            let session = session.clone();
            unsafe { session.RegisterAudioSessionNotification(Some(&self.event_handler))? };
            self.sessions.lock().push(session);
        }
        Ok(())
    }
}

pub struct MediaManager {
    sessions_manager: GlobalSystemMediaTransportControlsSessionManager,
}

impl MediaManager {
    /// T can be any type in the list:
    /// https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nf-mmdeviceapi-immdevice-activate
    pub fn activate_default_device<T: windows::core::Interface>() -> Result<T> {
        let enumerator: IMMDeviceEnumerator = Com::create_instance(&MMDeviceEnumerator)?;
        let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)? };
        let instance: T = unsafe { device.Activate(CLSCTX_ALL, None)? };
        Ok(instance)
    }

    pub fn new() -> Self {
        let controls_session_manager = tauri::async_runtime::block_on(async {
            GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
                .expect("Failed requesting transport controls")
                .await
                .expect("Failed requesting transport controls")
        });

        Self {
            sessions_manager: controls_session_manager,
        }
    }

    pub fn session_by_id(
        &self,
        id: &str,
    ) -> Result<Option<GlobalSystemMediaTransportControlsSession>> {
        for session in self.sessions_manager.GetSessions()? {
            if session.SourceAppUserModelId()?.to_string_lossy() == id {
                return Ok(Some(session));
            }
        }
        Ok(None)
    }

    pub async fn request_media_sessions(&self) -> Result<Vec<MediaSession>> {
        let mut sessions = Vec::new();

        let default_session = self.sessions_manager.GetCurrentSession()?;
        let default_session_id = default_session.SourceAppUserModelId()?.to_string_lossy();

        for session in self.sessions_manager.GetSessions()? {
            let properties = session.TryGetMediaPropertiesAsync()?.await?;

            let buffer = Buffer::Create(1_000_000)?; // 1MB
            let stream = properties.Thumbnail()?.OpenReadAsync()?.await?;
            stream
                .ReadAsync(&buffer, buffer.Capacity()?, InputStreamOptions::ReadAhead)?
                .await?;

            let reader = DataReader::FromBuffer(&buffer)?;
            let mut bytes = Vec::new();
            while reader.UnconsumedBufferLength()? > 0 {
                bytes.push(reader.ReadByte()?);
            }

            let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)?;
            let image_path = std::env::temp_dir().join(format!("{}.png", uuid::Uuid::new_v4()));
            image.save(&image_path)?;

            let playback_info = session.GetPlaybackInfo()?;
            let status = playback_info.PlaybackStatus()?;
            let id = session.SourceAppUserModelId()?.to_string_lossy();

            sessions.push(MediaSession {
                title: properties.Title()?.to_string_lossy(),
                author: properties.Artist()?.to_string_lossy(),
                thumbnail: Some(image_path),
                playing: status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
                default: id == default_session_id,
                id,
            });
        }

        Ok(sessions)
    }

    fn _listen_media_channels_events() {
        Com::run_threaded_with_context(move || -> Result<()> {
            let audio_session_manager: IAudioSessionManager2 = Self::activate_default_device()?;

            unsafe {
                let enumerator = audio_session_manager.GetSessionEnumerator()?;

                let handler: IAudioSessionNotification =
                    MediaSessionCreationEventHandler::new().into();
                audio_session_manager.RegisterSessionNotification(Some(&handler))?;

                let handler: IAudioSessionEvents = MediaSessionEventHandler.into();
                let mut sessions = Vec::new();

                for i in 0..enumerator.GetCount()? {
                    let session = enumerator.GetSession(i)?;
                    session.RegisterAudioSessionNotification(Some(&handler))?;
                    sessions.push(session);
                }

                while enumerator.GetCount().is_ok() {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }

            Ok(())
        });
    }

    pub fn listen_media_volume_events<F>(&self, callback: F)
    where
        F: Fn(f32, bool) + Send + 'static,
    {
        Com::run_threaded_with_context(move || -> Result<()> {
            let audio_endpoint: IAudioEndpointVolume = Self::activate_default_device()?;

            let callback: IAudioEndpointVolumeCallback =
                AudioEndpointVolumeCallback { callback }.into();
            unsafe { audio_endpoint.RegisterControlChangeNotify(&callback)? };

            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        });
    }

    pub fn listen_transport_controls_events<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let transport_session_manager = self.sessions_manager.clone();

        std::thread::spawn(move || -> Result<()> {
            let callback = Arc::new(callback);
            let callback_clone = Arc::clone(&callback);

            let register_sessions: Arc<dyn Fn() -> Result<()> + Send + Sync> =
                Arc::new(move || -> Result<()> {
                    let callback_clone = Arc::clone(&callback);
                    let property_changed = TypedEventHandler::new(move |_, _| {
                        callback_clone();
                        Ok(())
                    });

                    let callback_clone = Arc::clone(&callback);
                    let playback_info_changed = TypedEventHandler::new(move |_, _| {
                        callback_clone();
                        Ok(())
                    });
                    let controls_session_manager = tauri::async_runtime::block_on(
                        GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?,
                    )?;

                    for session in controls_session_manager.GetSessions()? {
                        let string_id = session.SourceAppUserModelId()?.to_string_lossy();

                        let mut property_dict = REG_PROPERTY_EVENTS.lock();
                        let mut playback_dict = REG_PLAYBACK_EVENTS.lock();

                        if let Some(token) = property_dict.get(&string_id) {
                            session.RemoveMediaPropertiesChanged(*token)?;
                        }

                        if let Some(token) = playback_dict.get(&string_id) {
                            session.RemoveMediaPropertiesChanged(*token)?;
                        }

                        property_dict.insert(
                            string_id.clone(),
                            session.MediaPropertiesChanged(&property_changed)?,
                        );
                        playback_dict.insert(
                            string_id.clone(),
                            session.PlaybackInfoChanged(&playback_info_changed)?,
                        );
                    }

                    Ok(())
                });

            // register initial sessions on startup
            register_sessions()?;

            // listen for futures changes in sessions
            let cloned_register_sessions = Arc::clone(&register_sessions);
            transport_session_manager.SessionsChanged(&TypedEventHandler::new(move |_, _| {
                log_error!(cloned_register_sessions());
                callback_clone();
                Ok(())
            }))?;

            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        });
    }
}
