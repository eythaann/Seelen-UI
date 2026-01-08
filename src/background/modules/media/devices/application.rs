use std::sync::LazyLock;

use windows::Win32::{
    Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
    Foundation::PROPERTYKEY,
    Media::Audio::{
        eAll, eCapture, eCommunications, eMultimedia, eRender, EDataFlow, ERole,
        Endpoints::{
            IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
        },
        IAudioSessionControl, IAudioSessionControl2, IAudioSessionEvents, IAudioSessionEvents_Impl,
        IAudioSessionManager2, IAudioSessionNotification, IAudioSessionNotification_Impl,
        IMMDevice, IMMDeviceEnumerator, IMMEndpoint, IMMNotificationClient,
        IMMNotificationClient_Impl, ISimpleAudioVolume, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    },
    System::Com::{CLSCTX_ALL, STGM_READ},
};
use windows_core::Interface;

use crate::{
    error::{ErrorMap, Result, ResultLogExt},
    event_manager,
    utils::{lock_free::SyncHashMap, pcwstr},
    windows_api::{process::Process, Com},
};

use super::domain::{MediaDevice, MediaDeviceSession, MediaDeviceType};

pub struct DevicesManager {
    inputs: SyncHashMap<String, MediaDevice>,
    outputs: SyncHashMap<String, MediaDevice>,

    device_enumerator: IMMDeviceEnumerator,
    mm_notification_client: IMMNotificationClient,
}

#[derive(Debug, Clone)]
pub enum DevicesEvent {
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
    SessionAdded {
        device_id: String,
        session: MediaDeviceSession,
    },
    SessionRemoved {
        device_id: String,
        session_id: String,
    },
    SessionVolumeChanged {
        device_id: String,
        session_id: String,
        volume: f32,
        muted: bool,
    },
}

unsafe impl Send for DevicesEvent {}

unsafe impl Send for DevicesManager {}
unsafe impl Sync for DevicesManager {}

event_manager!(DevicesManager, DevicesEvent);

impl DevicesManager {
    fn new() -> Result<Self> {
        Ok(Self {
            inputs: SyncHashMap::new(),
            outputs: SyncHashMap::new(),
            device_enumerator: Com::create_instance(&MMDeviceEnumerator)?,
            mm_notification_client: DevicesManagerEvents.into(),
        })
    }

    fn init(&mut self) -> Result<()> {
        unsafe {
            let collection = self
                .device_enumerator
                .EnumAudioEndpoints(eAll, DEVICE_STATE_ACTIVE)?;

            for idx in 0..collection.GetCount()? {
                // log instead propagate error to avoid panic if just some device fail to load
                self.load_device(&collection.Item(idx)?).log_error();
            }

            self.device_enumerator
                .RegisterEndpointNotificationCallback(&self.mm_notification_client)?;
        }

        let eid = Self::subscribe(|event| {
            DevicesManager::instance().process_event(event).log_error();
        });
        Self::set_event_handler_priority(&eid, 1);

        Ok(())
    }

    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<DevicesManager> = LazyLock::new(|| {
            let mut manager = DevicesManager::new().expect("Failed to create devices manager");
            manager.init().log_error();
            manager
        });
        &MANAGER
    }

    pub fn get_inputs(&self) -> Vec<MediaDevice> {
        self.inputs.values()
    }

    pub fn get_outputs(&self) -> Vec<MediaDevice> {
        self.outputs.values()
    }

    pub fn get_raw_device(&self, device_id: &str) -> Option<IMMDevice> {
        unsafe { self.device_enumerator.GetDevice(pcwstr(device_id)) }.ok()
    }

    pub fn set_volume_level(
        &self,
        device_id: String,
        session_id: Option<String>,
        mut level: f32,
    ) -> Result<()> {
        level = level.clamp(0.0, 1.0); // ensure valid value

        let cb = |device: &mut MediaDevice| {
            match &session_id {
                Some(session_id) => {
                    if let Some(session) = device.session(session_id) {
                        unsafe {
                            let volume: ISimpleAudioVolume = session.controls.cast()?;
                            volume.SetMasterVolume(level, &windows::core::GUID::zeroed())?;
                        }
                    }
                }
                None => unsafe {
                    device
                        .volume_endpoint
                        .SetMasterVolumeLevelScalar(level, &windows::core::GUID::zeroed())?;
                },
            }
            Ok(())
        };

        if let Some(result) = self.inputs.get(&device_id, cb) {
            return result;
        }
        if let Some(result) = self.outputs.get(&device_id, cb) {
            return result;
        }
        Ok(())
    }

    pub fn toggle_mute(&self, device_id: String, session_id: Option<String>) -> Result<()> {
        let cb = |device: &mut MediaDevice| {
            match &session_id {
                Some(session_id) => {
                    if let Some(session) = device.session(session_id) {
                        unsafe {
                            let volume: ISimpleAudioVolume = session.controls.cast()?;
                            volume.SetMute(
                                !volume.GetMute()?.as_bool(),
                                &windows::core::GUID::zeroed(),
                            )?;
                        }
                    }
                }
                None => unsafe {
                    device.volume_endpoint.SetMute(
                        !device.volume_endpoint.GetMute()?.as_bool(),
                        &windows::core::GUID::zeroed(),
                    )?;
                },
            }
            Ok(())
        };

        if let Some(result) = self.inputs.get(&device_id, cb) {
            return result;
        }
        if let Some(result) = self.outputs.get(&device_id, cb) {
            return result;
        }
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

    fn process_event(&self, event: DevicesEvent) -> Result<()> {
        match &event {
            DevicesEvent::DeviceAdded(device_id) => {
                if let Some(device) = self.get_raw_device(device_id) {
                    self.load_device(&device)?;
                }
            }
            DevicesEvent::DeviceRemoved(device_id) => {
                self.remove_device(device_id);
            }
            DevicesEvent::DefaultDeviceChanged {
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
            DevicesEvent::DeviceVolumeChanged {
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
            DevicesEvent::SessionAdded { device_id, session } => {
                let cb = |device: &mut MediaDevice| {
                    device.sessions.push(session.clone());
                };
                self.inputs.get(device_id, cb);
                self.outputs.get(device_id, cb);
            }
            DevicesEvent::SessionRemoved {
                device_id,
                session_id,
            } => {
                let cb = |device: &mut MediaDevice| {
                    device.remove_session(session_id);
                };
                self.inputs.get(device_id, cb);
                self.outputs.get(device_id, cb);
            }
            DevicesEvent::SessionVolumeChanged {
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
        }
        Ok(())
    }
}

impl MediaDevice {
    pub unsafe fn load(raw_device: &IMMDevice) -> Result<Self> {
        let device_id = raw_device.GetId()?.to_string()?;
        let volume_endpoint: IAudioEndpointVolume = raw_device.Activate(CLSCTX_ALL, None)?;
        let session_manager: IAudioSessionManager2 = raw_device.Activate(CLSCTX_ALL, None)?;

        let mut sessions = Vec::new();
        let enumerator = session_manager.GetSessionEnumerator()?;
        for session_idx in 0..enumerator.GetCount()? {
            let session: IAudioSessionControl2 = enumerator.GetSession(session_idx)?.cast()?;
            match MediaDeviceSession::load(session, &device_id) {
                Ok(session) => sessions.push(session),
                Err(e) => log::error!("Failed to load session: {e:?}"),
            }
        }

        let properties = raw_device.OpenPropertyStore(STGM_READ)?;
        let data_flow = if raw_device.cast::<IMMEndpoint>()?.GetDataFlow()? == eCapture {
            MediaDeviceType::Input
        } else {
            MediaDeviceType::Output
        };

        let volume_callback = IAudioEndpointVolumeCallback::from(MediaDeviceEventHandler {
            device_id: device_id.clone(),
        });

        let session_created_callback = IAudioSessionNotification::from(MediaDeviceEventHandler {
            device_id: device_id.clone(),
        });

        let device = MediaDevice {
            id: device_id.clone(),
            name: properties.GetValue(&PKEY_Device_FriendlyName)?.to_string(),
            r#type: data_flow,
            is_default_multimedia: false, // unset, parent should set this
            is_default_communications: false, // unset, parent should set this
            sessions,
            volume: volume_endpoint.GetMasterVolumeLevelScalar()?,
            muted: volume_endpoint.GetMute()?.as_bool(),
            volume_endpoint,
            volume_callback,
            session_manager,
            session_created_callback,
        };

        device
            .volume_endpoint
            .RegisterControlChangeNotify(&device.volume_callback)?;
        device
            .session_manager
            .RegisterSessionNotification(&device.session_created_callback)?;
        Ok(device)
    }
}

impl MediaDeviceSession {
    pub unsafe fn load(session: IAudioSessionControl2, device_id: &str) -> Result<Self> {
        let session_id = session.GetSessionIdentifier()?.to_string()?;
        let volume: ISimpleAudioVolume = session.cast()?;
        let proccess = Process::from_id(session.GetProcessId()?);

        let events_callback = IAudioSessionEvents::from(MediaSessionEventHandler {
            device_id: device_id.to_owned(),
            session_id: session_id.clone(),
        });

        let session = MediaDeviceSession {
            id: session_id,
            instance_id: session.GetSessionInstanceIdentifier()?.to_string()?,
            process_id: proccess.id(),
            name: proccess
                .program_display_name()
                .unwrap_or_else(|_| "???".to_string()),
            icon_path: proccess.program_path().ok(),
            is_system: session.IsSystemSoundsSession().0 == 0,
            volume: volume.GetMasterVolume()?,
            muted: volume.GetMute()?.as_bool(),
            controls: session,
            events_callback,
        };

        session
            .controls
            .RegisterAudioSessionNotification(&session.events_callback)?;
        Ok(session)
    }
}

impl Drop for DevicesManager {
    fn drop(&mut self) {
        self.inputs.clear();
        self.outputs.clear();
        unsafe {
            self.device_enumerator
                .UnregisterEndpointNotificationCallback(&self.mm_notification_client)
                .log_error();
        }
    }
}

#[windows_core::implement(IMMNotificationClient)]
struct DevicesManagerEvents;

impl IMMNotificationClient_Impl for DevicesManagerEvents_Impl {
    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        device_id: &windows_core::PCWSTR,
    ) -> windows_core::Result<()> {
        DevicesManager::send(DevicesEvent::DefaultDeviceChanged {
            flow,
            role,
            device_id: unsafe { device_id.to_string()? },
        });
        Ok(())
    }

    fn OnDeviceAdded(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        DevicesManager::send(DevicesEvent::DeviceAdded(unsafe { device_id.to_string()? }));
        Ok(())
    }

    fn OnDeviceRemoved(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        DevicesManager::send(DevicesEvent::DeviceRemoved(unsafe {
            device_id.to_string()?
        }));
        Ok(())
    }

    fn OnDeviceStateChanged(
        &self,
        device_id: &windows_core::PCWSTR,
        new_device_state: windows::Win32::Media::Audio::DEVICE_STATE,
    ) -> windows_core::Result<()> {
        let device_id = unsafe { device_id.to_string()? };
        let tx = DevicesManager::event_tx();
        match new_device_state {
            DEVICE_STATE_ACTIVE => tx.send(DevicesEvent::DeviceAdded(device_id)),
            _ => tx.send(DevicesEvent::DeviceRemoved(device_id)),
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

#[windows::core::implement(IAudioEndpointVolumeCallback, IAudioSessionNotification)]
pub struct MediaDeviceEventHandler {
    device_id: String,
}

impl IAudioEndpointVolumeCallback_Impl for MediaDeviceEventHandler_Impl {
    fn OnNotify(
        &self,
        data: *mut windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA,
    ) -> windows_core::Result<()> {
        if let Some(data) = unsafe { data.as_ref() } {
            let tx = DevicesManager::event_tx();
            let result = tx.send(DevicesEvent::DeviceVolumeChanged {
                device_id: self.device_id.clone(),
                volume: data.fMasterVolume,
                muted: data.bMuted.as_bool(),
            });
            result.log_error();
        }
        Ok(())
    }
}

impl IAudioSessionNotification_Impl for MediaDeviceEventHandler_Impl {
    fn OnSessionCreated(
        &self,
        new_session: windows_core::Ref<'_, IAudioSessionControl>,
    ) -> windows_core::Result<()> {
        if let Some(new_session) = new_session.as_ref() {
            let new_session: IAudioSessionControl2 = new_session.cast()?;
            match unsafe { MediaDeviceSession::load(new_session, &self.device_id) } {
                Ok(session) => {
                    let tx = DevicesManager::event_tx();
                    tx.send(DevicesEvent::SessionAdded {
                        device_id: self.device_id.clone(),
                        session,
                    })
                    .log_error();
                }
                Err(e) => log::error!("Failed to load session: {e:?}"),
            }
        }
        Ok(())
    }
}

#[windows_core::implement(IAudioSessionEvents)]
pub struct MediaSessionEventHandler {
    device_id: String,
    session_id: String,
}

impl IAudioSessionEvents_Impl for MediaSessionEventHandler_Impl {
    fn OnChannelVolumeChanged(
        &self,
        _channel_count: u32,
        _new_channel_volume_array: *const f32,
        _changed_channel: u32,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnDisplayNameChanged(
        &self,
        _new_display_name: &windows::core::PCWSTR,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnGroupingParamChanged(
        &self,
        _new_grouping_param: *const windows::core::GUID,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnIconPathChanged(
        &self,
        _new_icon_path: &windows::core::PCWSTR,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnSessionDisconnected(
        &self,
        _disconnect_reason: windows::Win32::Media::Audio::AudioSessionDisconnectReason,
    ) -> windows::core::Result<()> {
        let tx = DevicesManager::event_tx();
        let result = tx.send(DevicesEvent::SessionRemoved {
            device_id: self.device_id.clone(),
            session_id: self.session_id.clone(),
        });
        result.log_error();
        Ok(())
    }

    fn OnSimpleVolumeChanged(
        &self,
        new_volume: f32,
        new_mute: windows::Win32::Foundation::BOOL,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        let tx = DevicesManager::event_tx();
        let result = tx.send(DevicesEvent::SessionVolumeChanged {
            device_id: self.device_id.clone(),
            session_id: self.session_id.clone(),
            volume: new_volume,
            muted: new_mute.as_bool(),
        });
        result.log_error();
        Ok(())
    }

    fn OnStateChanged(
        &self,
        _new_state: windows::Win32::Media::Audio::AudioSessionState,
    ) -> windows::core::Result<()> {
        Ok(())
    }
}
