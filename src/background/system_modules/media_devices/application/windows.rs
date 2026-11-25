use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use seelen_core::system_state::{MediaDevice, MediaDeviceSession, MediaDeviceType};
use windows::Win32::{
    Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
    Foundation::PROPERTYKEY,
    Media::Audio::{
        AUDIO_VOLUME_NOTIFICATION_DATA, AudioSessionDisconnectReason, AudioSessionState,
        AudioSessionStateExpired, DEVICE_STATE, DEVICE_STATE_ACTIVE, EDataFlow, ERole,
        Endpoints::{
            IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
        },
        IAudioSessionControl, IAudioSessionControl2, IAudioSessionEvents, IAudioSessionEvents_Impl,
        IAudioSessionManager2, IAudioSessionNotification, IAudioSessionNotification_Impl,
        IMMDevice, IMMDeviceEnumerator, IMMEndpoint, IMMNotificationClient,
        IMMNotificationClient_Impl, ISimpleAudioVolume, MMDeviceEnumerator, eAll, eCapture,
        eCommunications, eMultimedia, eRender,
    },
    System::Com::{CLSCTX_ALL, STGM_READ},
};
use windows_core::{BOOL, GUID, Interface, PCWSTR};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    platform::process::{Process, WindowsProcessExt},
    system_modules::media_devices::domain::{MediaDevicesSysEvent, MediaDevicesSystem},
    utils::lock_free::SyncHashMap,
    windows_api::{Com, ComThread, WindowsApi, string_utils::WindowsString},
};

pub struct WindowsMediaDevices {
    state: Option<Arc<DevicesState>>,
    /// Subscription of `state` to the raw COM notifications.
    subscription: Option<String>,
}

impl MediaDevicesSystem for WindowsMediaDevices {
    fn create() -> Self {
        Self {
            state: None,
            subscription: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(MediaDevicesSysEvent) + Send + Sync + 'static,
    {
        let state = Arc::new(DevicesState::create()?);
        self.state = Some(state.clone());

        // The COM callbacks only queue the raw notifications: querying COM from inside them
        // can deadlock, so they are applied to the state from the event_manager thread.
        let processor = state.clone();
        self.subscription = Some(DevicesState::subscribe(move |event| {
            processor.process_event(&event).log_error();
            on_event(match event {
                RawDevicesEvent::DefaultDeviceChanged { flow, role, .. }
                    if flow == eRender && role == eMultimedia =>
                {
                    MediaDevicesSysEvent::DefaultOutputChanged
                }
                _ => MediaDevicesSysEvent::Changed,
            });
        }));

        state.load_devices()
    }

    fn inputs(&self) -> Vec<MediaDevice> {
        self.state
            .as_ref()
            .map(|s| s.inputs.map(|(_, device)| device.to_core()))
            .unwrap_or_default()
    }

    fn outputs(&self) -> Vec<MediaDevice> {
        self.state
            .as_ref()
            .map(|s| s.outputs.map(|(_, device)| device.to_core()))
            .unwrap_or_default()
    }

    fn set_default_device(&self, device_id: &str, role: &str) -> Result<()> {
        WindowsApi::set_default_audio_device(device_id, role)
    }

    fn toggle_mute(&self, device_id: &str, session_id: Option<&str>) -> Result<()> {
        self.with_device(device_id, |device| unsafe {
            match session_id {
                Some(session_id) => {
                    let volumes = device.app_volumes(session_id);
                    // mute the whole app unless all its sessions are already muted
                    let mute = volumes
                        .iter()
                        .any(|volume| volume.GetMute().is_ok_and(|muted| !muted.as_bool()));
                    for volume in volumes {
                        volume.SetMute(mute, &GUID::zeroed()).log_error();
                    }
                }
                None => {
                    let endpoint = &device.volume_endpoint;
                    endpoint.SetMute(!endpoint.GetMute()?.as_bool(), &GUID::zeroed())?;
                }
            }
            Ok(())
        })
    }

    fn set_volume_level(
        &self,
        device_id: &str,
        session_id: Option<&str>,
        level: f32,
    ) -> Result<()> {
        self.with_device(device_id, |device| unsafe {
            match session_id {
                Some(session_id) => {
                    for volume in device.app_volumes(session_id) {
                        volume.SetMasterVolume(level, &GUID::zeroed()).log_error();
                    }
                }
                None => {
                    device
                        .volume_endpoint
                        .SetMasterVolumeLevelScalar(level, &GUID::zeroed())?;
                }
            }
            Ok(())
        })
    }
}

impl WindowsMediaDevices {
    /// Runs `f` on the device with that id. Unknown devices are ignored.
    fn with_device(
        &self,
        device_id: &str,
        f: impl Fn(&mut NativeDevice) -> Result<()>,
    ) -> Result<()> {
        let Some(state) = &self.state else {
            return Err("Media devices not initialized".into());
        };
        let device_id = device_id.to_owned();
        if let Some(result) = state.inputs.get(&device_id, &f) {
            return result;
        }
        if let Some(result) = state.outputs.get(&device_id, &f) {
            return result;
        }
        Ok(())
    }
}

impl Drop for WindowsMediaDevices {
    fn drop(&mut self) {
        if let Some(id) = self.subscription.take() {
            DevicesState::unsubscribe(&id);
        }
        if let Some(state) = self.state.take() {
            state.release();
        }
    }
}

/// Marks a COM interface pointer as movable across threads. Sound as long as no method is
/// ever called on it from more than one thread at a time — true here: values wrapped in
/// this only ever cross from the devices COM thread back to whichever thread asked for
/// them, and from then on only that thread touches them.
struct SendCom<T>(T);
unsafe impl<T> Send for SendCom<T> {}

struct DevicesComState {
    device_enumerator: IMMDeviceEnumerator,
    mm_notification_client: IMMNotificationClient,
}

/// Loaded devices, shared with the processor of the raw notifications.
struct DevicesState {
    inputs: SyncHashMap<String, NativeDevice>,
    outputs: SyncHashMap<String, NativeDevice>,
    com: ComThread<DevicesComState>,
}

/// Raw notifications of the COM callbacks, applied to [`DevicesState`] out of the callbacks.
#[derive(Debug, Clone)]
enum RawDevicesEvent {
    DeviceAdded {
        device_id: String,
    },
    DeviceRemoved {
        device_id: String,
    },
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
        session: NativeSession,
    },
    SessionRemoved {
        device_id: String,
        instance_id: String,
    },
    SessionVolumeChanged {
        device_id: String,
        instance_id: String,
        volume: f32,
        muted: bool,
    },
}

// SAFETY: `NativeSession` holds COM pointers that are only used by the processor thread.
unsafe impl Send for RawDevicesEvent {}

// SAFETY: the COM pointers of the devices are only used through the maps' locks.
unsafe impl Send for DevicesState {}
unsafe impl Sync for DevicesState {}

event_manager!(DevicesState, RawDevicesEvent);

impl DevicesState {
    fn create() -> Result<Self> {
        let com = ComThread::spawn("Devices COM", || {
            Ok(DevicesComState {
                device_enumerator: Com::create_instance(&MMDeviceEnumerator)?,
                mm_notification_client: DevicesNotificationClient.into(),
            })
        })?;
        Ok(Self {
            inputs: SyncHashMap::new(),
            outputs: SyncHashMap::new(),
            com,
        })
    }

    /// Loads the active devices and registers for their changes.
    fn load_devices(&self) -> Result<()> {
        let devices = self.com.call(|state| unsafe {
            let collection = state
                .device_enumerator
                .EnumAudioEndpoints(eAll, DEVICE_STATE_ACTIVE)?;

            let mut devices = Vec::new();
            for idx in 0..collection.GetCount()? {
                devices.push(SendCom(collection.Item(idx)?));
            }

            state
                .device_enumerator
                .RegisterEndpointNotificationCallback(&state.mm_notification_client)?;

            Ok(devices)
        })?;

        for device in devices {
            // log instead propagate error so a failing device doesn't hide the others
            self.load_device(&device.0).log_error();
        }
        Ok(())
    }

    fn release(&self) {
        for (_, device) in self.inputs.take().into_iter().chain(self.outputs.take()) {
            device.release();
        }
        self.com
            .call(|state| {
                unsafe {
                    state
                        .device_enumerator
                        .UnregisterEndpointNotificationCallback(&state.mm_notification_client)
                }
                .log_error();
                Ok(())
            })
            .log_error();
    }

    fn raw_device(&self, device_id: &str) -> Option<IMMDevice> {
        let device_id = device_id.to_owned();
        self.com
            .call(move |state| {
                let device_id = WindowsString::from_str(&device_id);
                Ok(SendCom(
                    unsafe { state.device_enumerator.GetDevice(device_id.as_pcwstr()) }.ok(),
                ))
            })
            .ok()
            .and_then(|SendCom(device)| device)
    }

    fn load_device(&self, device: &IMMDevice) -> Result<()> {
        let mut device = unsafe { NativeDevice::load(device)? };
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

    fn is_default_device(&self, device: &NativeDevice, role: ERole) -> bool {
        let dataflow = match device.r#type {
            MediaDeviceType::Input => eCapture,
            MediaDeviceType::Output => eRender,
        };
        let device_id = device.id.clone();
        self.com
            .call(move |state| unsafe {
                Ok(state
                    .device_enumerator
                    .GetDefaultAudioEndpoint(dataflow, role)
                    .and_then(|d| d.GetId())
                    .map(|id| id.to_hstring() == device_id)
                    .unwrap_or(false))
            })
            .unwrap_or(false)
    }

    /// Runs `f` on the device with that id, wherever it is.
    fn update_device(&self, device_id: &str, f: impl Fn(&mut NativeDevice)) {
        self.inputs.get(device_id, &f);
        self.outputs.get(device_id, &f);
    }

    fn process_event(&self, event: &RawDevicesEvent) -> Result<()> {
        match event {
            RawDevicesEvent::DeviceAdded { device_id } => {
                if let Some(device) = self.raw_device(device_id) {
                    self.load_device(&device)?;
                }
            }
            RawDevicesEvent::DeviceRemoved { device_id } => {
                self.remove_device(device_id);
            }
            RawDevicesEvent::DefaultDeviceChanged {
                flow,
                role,
                device_id,
            } => {
                let devices = if *flow == eCapture {
                    &self.inputs
                } else {
                    &self.outputs
                };

                // Windows can report a new default device before we have loaded it,
                // e.g. a bluetooth endpoint that is still being enumerated. Flagging
                // blindly would then clear the flag on every device and set it on
                // none, leaving the UI with no default device at all.
                if !device_id.is_empty()
                    && !devices.contains_key(device_id)
                    && let Some(raw) = self.raw_device(device_id)
                {
                    self.load_device(&raw).log_error();
                }

                devices.for_each(|(_, device)| {
                    if *role == eMultimedia {
                        device.is_default_multimedia = device.id == *device_id;
                    } else if *role == eCommunications {
                        device.is_default_communications = device.id == *device_id;
                    }
                });
            }
            RawDevicesEvent::DeviceVolumeChanged {
                device_id,
                volume,
                muted,
            } => {
                self.update_device(device_id, |device| {
                    device.volume = *volume;
                    device.muted = *muted;
                });
            }
            RawDevicesEvent::SessionAdded { device_id, session } => {
                self.update_device(device_id, |device| {
                    // a session can be reported twice when it is created while loading
                    device.remove_session(&session.instance_id);
                    device.sessions.push(session.clone());
                });
            }
            RawDevicesEvent::SessionRemoved {
                device_id,
                instance_id,
            } => {
                self.update_device(device_id, |device| {
                    device.remove_session(instance_id);
                });
            }
            RawDevicesEvent::SessionVolumeChanged {
                device_id,
                instance_id,
                volume,
                muted,
            } => {
                self.update_device(device_id, |device| {
                    if let Some(session) = device.session_mut(instance_id) {
                        session.volume = *volume;
                        session.muted = *muted;
                    }
                });
            }
        }
        Ok(())
    }
}

/// An audio endpoint and the COM objects watching it.
#[derive(Debug, Clone)]
struct NativeDevice {
    volume_endpoint: IAudioEndpointVolume,
    volume_callback: IAudioEndpointVolumeCallback,
    session_manager: Option<IAudioSessionManager2>,
    session_created_callback: Option<IAudioSessionNotification>,
    // ---
    id: String,
    name: String,
    r#type: MediaDeviceType,
    is_default_multimedia: bool,
    is_default_communications: bool,
    sessions: Vec<NativeSession>,
    volume: f32,
    muted: bool,
}

type SessionManagerData = (
    Vec<NativeSession>,
    Option<IAudioSessionManager2>,
    Option<IAudioSessionNotification>,
);

impl NativeDevice {
    unsafe fn load(raw_device: &IMMDevice) -> Result<Self> {
        unsafe {
            let device_id = WindowsString::from(raw_device.GetId()?).to_string();
            let volume_endpoint: IAudioEndpointVolume = raw_device.Activate(CLSCTX_ALL, None)?;
            let (sessions, session_manager, session_created_callback) =
                Self::load_sessions(raw_device, &device_id);

            let properties = raw_device.OpenPropertyStore(STGM_READ)?;
            let data_flow = if raw_device.cast::<IMMEndpoint>()?.GetDataFlow()? == eCapture {
                MediaDeviceType::Input
            } else {
                MediaDeviceType::Output
            };

            let volume_callback = IAudioEndpointVolumeCallback::from(DeviceEventHandler {
                device_id: device_id.clone(),
            });

            let device = NativeDevice {
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
            Ok(device)
        }
    }

    // Session manager activation may fail for some devices (HDMI without display,
    // Bluetooth, virtual devices). Returns empty sessions so the device still appears.
    unsafe fn load_sessions(raw_device: &IMMDevice, device_id: &str) -> SessionManagerData {
        unsafe {
            match Self::try_load_sessions(raw_device, device_id) {
                Ok(data) => data,
                Err(e) => {
                    log::warn!(
                        "Session manager unavailable for device {device_id}, sessions will not be tracked: {e:?}"
                    );
                    (Vec::new(), None, None)
                }
            }
        }
    }

    unsafe fn try_load_sessions(
        raw_device: &IMMDevice,
        device_id: &str,
    ) -> Result<SessionManagerData> {
        unsafe {
            let sm: IAudioSessionManager2 = raw_device.Activate(CLSCTX_ALL, None)?;
            let mut sessions = Vec::new();
            let enumerator = sm.GetSessionEnumerator()?;
            for session_idx in 0..enumerator.GetCount()? {
                let session: IAudioSessionControl2 = enumerator.GetSession(session_idx)?.cast()?;
                if session
                    .GetState()
                    .is_ok_and(|state| state == AudioSessionStateExpired)
                {
                    continue;
                }
                match NativeSession::load(session, device_id) {
                    Ok(s) => sessions.push(s),
                    Err(e) => log::error!("Failed to load session: {e:?}"),
                }
            }
            let cb = IAudioSessionNotification::from(DeviceEventHandler {
                device_id: device_id.to_owned(),
            });
            sm.RegisterSessionNotification(&cb)?;
            Ok((sessions, Some(sm), Some(cb)))
        }
    }

    fn to_core(&self) -> MediaDevice {
        MediaDevice {
            id: self.id.clone(),
            name: self.name.clone(),
            r#type: self.r#type,
            is_default_multimedia: self.is_default_multimedia,
            is_default_communications: self.is_default_communications,
            sessions: self.sessions.iter().map(|s| s.to_core()).collect(),
            volume: self.volume,
            muted: self.muted,
        }
    }

    /// Volume controls of every session of the app owning the session `session_id`
    /// (its `instance_id`, or its `id` for callers that still send it).
    fn app_volumes(&self, session_id: &str) -> Vec<ISimpleAudioVolume> {
        let Some(target) = self
            .sessions
            .iter()
            .find(|s| s.instance_id == session_id)
            .or_else(|| self.sessions.iter().find(|s| s.id == session_id))
        else {
            return Vec::new();
        };
        self.sessions
            .iter()
            .filter(|s| s.app_id == target.app_id)
            .filter_map(|s| s.controls.cast().ok())
            .collect()
    }

    fn session_mut(&mut self, instance_id: &str) -> Option<&mut NativeSession> {
        self.sessions
            .iter_mut()
            .find(|s| s.instance_id == instance_id)
    }

    fn remove_session(&mut self, instance_id: &str) {
        for session in std::mem::take(&mut self.sessions) {
            if session.instance_id == instance_id {
                session.release();
                continue;
            }
            self.sessions.push(session);
        }
    }

    fn release(&self) {
        unsafe {
            self.volume_endpoint
                .UnregisterControlChangeNotify(&self.volume_callback)
                .log_error();
            if let (Some(sm), Some(cb)) = (&self.session_manager, &self.session_created_callback) {
                sm.UnregisterSessionNotification(cb).log_error();
            }
        };
    }
}

/// An audio session of a device and the COM objects watching it.
#[derive(Debug, Clone)]
struct NativeSession {
    controls: IAudioSessionControl2,
    events_callback: IAudioSessionEvents,
    // ---
    id: String,
    instance_id: String,
    app_id: String,
    process_id: u32,
    name: String,
    icon_path: Option<PathBuf>,
    is_system: bool,
    volume: f32,
    muted: bool,
}

impl NativeSession {
    unsafe fn load(session: IAudioSessionControl2, device_id: &str) -> Result<Self> {
        unsafe {
            let session_id = WindowsString::from(session.GetSessionIdentifier()?).to_string();
            let instance_id =
                WindowsString::from(session.GetSessionInstanceIdentifier()?).to_string();
            let volume: ISimpleAudioVolume = session.cast()?;
            let process = Process::from_id(session.GetProcessId()?);
            let is_system = session.IsSystemSoundsSession().0 == 0;

            let events_callback = IAudioSessionEvents::from(SessionEventHandler {
                device_id: device_id.to_owned(),
                instance_id: instance_id.clone(),
            });

            let session = NativeSession {
                app_id: Self::app_id(&process, is_system, &session_id),
                id: session_id,
                instance_id,
                process_id: process.id(),
                name: process.display_name().unwrap_or_else(|_| "???".to_string()),
                icon_path: process.exe_path().ok(),
                is_system,
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

    /// Groups the sessions by app: all processes of a packaged app share their package, the
    /// others their executable and the process that launched that executable (e.g. Discord's
    /// voice process is a child of its main process). The root process keeps apart unrelated
    /// apps sharing a host executable (WebView2 apps, `java.exe`, `python.exe`...). Windows'
    /// own session identifier is not enough, as the processes of a same app can get different
    /// ones.
    fn app_id(process: &Process, is_system: bool, session_id: &str) -> String {
        if is_system {
            return "system".to_owned();
        }
        if let Ok(aumid) = process.package_app_user_model_id() {
            // "PackageFamilyName!AppId"
            let family = aumid.split('!').next().unwrap_or(&aumid);
            return format!("package:{}", family.to_lowercase());
        }
        if let Ok(path) = process.exe_path() {
            let root = Self::root_process_id(process, &path);
            return format!("exe:{}|{root}", path.to_string_lossy().to_lowercase());
        }
        // the process can't be inspected (e.g. elevated): Windows' identifier still names its
        // executable, which is at least shared by all its processes
        match exe_from_session_identifier(session_id) {
            Some(exe) => format!("exe:{}", exe.to_lowercase()),
            None => session_id.to_owned(),
        }
    }

    /// Id of the first process of the tree of processes running the executable `exe`.
    fn root_process_id(process: &Process, exe: &Path) -> u32 {
        let mut root = Process::from_id(process.id());
        // bounded, in case of a cycle caused by reused ids
        for _ in 0..32 {
            match root.parent() {
                Ok(parent) if parent.exe_path().is_ok_and(|path| path == exe) => root = parent,
                _ => break,
            }
        }
        root.id()
    }

    fn to_core(&self) -> MediaDeviceSession {
        MediaDeviceSession {
            id: self.id.clone(),
            instance_id: self.instance_id.clone(),
            app_id: self.app_id.clone(),
            process_id: self.process_id,
            name: self.name.clone(),
            icon_path: self.icon_path.clone(),
            is_system: self.is_system,
            volume: self.volume,
            muted: self.muted,
        }
    }

    fn release(self) {
        unsafe {
            self.controls
                .UnregisterAudioSessionNotification(&self.events_callback)
                .log_error();
        }
    }
}

#[windows_core::implement(IMMNotificationClient)]
struct DevicesNotificationClient;

impl IMMNotificationClient_Impl for DevicesNotificationClient_Impl {
    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        device_id: &PCWSTR,
    ) -> windows_core::Result<()> {
        DevicesState::send(RawDevicesEvent::DefaultDeviceChanged {
            flow,
            role,
            device_id: WindowsString::from(*device_id).to_string(),
        });
        Ok(())
    }

    fn OnDeviceAdded(&self, device_id: &PCWSTR) -> windows_core::Result<()> {
        DevicesState::send(RawDevicesEvent::DeviceAdded {
            device_id: WindowsString::from(*device_id).to_string(),
        });
        Ok(())
    }

    fn OnDeviceRemoved(&self, device_id: &PCWSTR) -> windows_core::Result<()> {
        DevicesState::send(RawDevicesEvent::DeviceRemoved {
            device_id: WindowsString::from(*device_id).to_string(),
        });
        Ok(())
    }

    fn OnDeviceStateChanged(
        &self,
        device_id: &PCWSTR,
        new_device_state: DEVICE_STATE,
    ) -> windows_core::Result<()> {
        let device_id = WindowsString::from(*device_id).to_string();
        DevicesState::send(match new_device_state {
            DEVICE_STATE_ACTIVE => RawDevicesEvent::DeviceAdded { device_id },
            _ => RawDevicesEvent::DeviceRemoved { device_id },
        });
        Ok(())
    }

    fn OnPropertyValueChanged(
        &self,
        _device_id: &PCWSTR,
        _key: &PROPERTYKEY,
    ) -> windows_core::Result<()> {
        Ok(())
    }
}

#[windows_core::implement(IAudioEndpointVolumeCallback, IAudioSessionNotification)]
struct DeviceEventHandler {
    device_id: String,
}

impl IAudioEndpointVolumeCallback_Impl for DeviceEventHandler_Impl {
    fn OnNotify(&self, data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows_core::Result<()> {
        if let Some(data) = unsafe { data.as_ref() } {
            DevicesState::send(RawDevicesEvent::DeviceVolumeChanged {
                device_id: self.device_id.clone(),
                volume: data.fMasterVolume,
                muted: data.bMuted.as_bool(),
            });
        }
        Ok(())
    }
}

impl IAudioSessionNotification_Impl for DeviceEventHandler_Impl {
    fn OnSessionCreated(
        &self,
        new_session: windows_core::Ref<'_, IAudioSessionControl>,
    ) -> windows_core::Result<()> {
        if let Some(new_session) = new_session.as_ref() {
            let new_session: IAudioSessionControl2 = new_session.cast()?;
            match unsafe { NativeSession::load(new_session, &self.device_id) } {
                Ok(session) => DevicesState::send(RawDevicesEvent::SessionAdded {
                    device_id: self.device_id.clone(),
                    session,
                }),
                Err(e) => log::error!("Failed to load session: {e:?}"),
            }
        }
        Ok(())
    }
}

#[windows_core::implement(IAudioSessionEvents)]
struct SessionEventHandler {
    device_id: String,
    instance_id: String,
}

impl IAudioSessionEvents_Impl for SessionEventHandler_Impl {
    fn OnChannelVolumeChanged(
        &self,
        _channel_count: u32,
        _new_channel_volume_array: *const f32,
        _changed_channel: u32,
        _event_context: *const GUID,
    ) -> windows_core::Result<()> {
        Ok(())
    }

    fn OnDisplayNameChanged(
        &self,
        _new_display_name: &PCWSTR,
        _event_context: *const GUID,
    ) -> windows_core::Result<()> {
        Ok(())
    }

    fn OnGroupingParamChanged(
        &self,
        _new_grouping_param: *const GUID,
        _event_context: *const GUID,
    ) -> windows_core::Result<()> {
        Ok(())
    }

    fn OnIconPathChanged(
        &self,
        _new_icon_path: &PCWSTR,
        _event_context: *const GUID,
    ) -> windows_core::Result<()> {
        Ok(())
    }

    fn OnSessionDisconnected(
        &self,
        _disconnect_reason: AudioSessionDisconnectReason,
    ) -> windows_core::Result<()> {
        DevicesState::send(RawDevicesEvent::SessionRemoved {
            device_id: self.device_id.clone(),
            instance_id: self.instance_id.clone(),
        });
        Ok(())
    }

    fn OnSimpleVolumeChanged(
        &self,
        new_volume: f32,
        new_mute: BOOL,
        _event_context: *const GUID,
    ) -> windows_core::Result<()> {
        DevicesState::send(RawDevicesEvent::SessionVolumeChanged {
            device_id: self.device_id.clone(),
            instance_id: self.instance_id.clone(),
            volume: new_volume,
            muted: new_mute.as_bool(),
        });
        Ok(())
    }

    fn OnStateChanged(&self, new_state: AudioSessionState) -> windows_core::Result<()> {
        // an expired session will never play again (its process released it or exited), but
        // it is not always disconnected: keeping it would distort its app's row in the mixer
        if new_state == AudioSessionStateExpired {
            DevicesState::send(RawDevicesEvent::SessionRemoved {
                device_id: self.device_id.clone(),
                instance_id: self.instance_id.clone(),
            });
        }
        Ok(())
    }
}

/// Executable part of a session identifier, e.g.
/// `{0.0.0.00000000}.{guid}|\Device\HarddiskVolume3\Program Files\App\app.exe%b{guid}`.
fn exe_from_session_identifier(session_id: &str) -> Option<&str> {
    let (_, rest) = session_id.split_once('|')?;
    let exe = rest.split_once("%b").map_or(rest, |(exe, _)| exe);
    (!exe.is_empty()).then_some(exe)
}

#[cfg(test)]
mod tests {
    use super::exe_from_session_identifier;

    #[test]
    fn exe_is_read_from_session_identifier() {
        assert_eq!(
            exe_from_session_identifier(
                r"{0.0.0.00000000}.{a1b2}|\Device\HarddiskVolume3\Apps\Discord.exe%b{00000000-0000-0000-0000-000000000000}"
            ),
            Some(r"\Device\HarddiskVolume3\Apps\Discord.exe")
        );
        assert_eq!(
            exe_from_session_identifier(r"{0.0.0.00000000}.{a1b2}|\Device\app.exe"),
            Some(r"\Device\app.exe")
        );
    }

    #[test]
    fn identifier_without_exe() {
        assert_eq!(
            exe_from_session_identifier("{0.0.0.00000000}.{a1b2}|%b{guid}"),
            None
        );
        assert_eq!(exe_from_session_identifier("no-separator"), None);
    }
}
