use windows::Win32::{
    Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
    Media::Audio::{
        eCapture,
        Endpoints::{
            IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
        },
        IAudioSessionControl, IAudioSessionControl2, IAudioSessionManager2,
        IAudioSessionNotification, IAudioSessionNotification_Impl, IMMDevice, IMMEndpoint,
    },
    System::Com::{CLSCTX_ALL, STGM_READ},
};
use windows_core::Interface;

use crate::{
    error::Result,
    log_error,
    modules::media::domain::{MediaDevice, MediaDeviceSession, MediaDeviceType},
};

use super::{MediaEvent, MediaManager};

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
        new_session: windows_core::Ref<'_, IAudioSessionControl>,
    ) -> windows_core::Result<()> {
        if let Some(new_session) = new_session.as_ref() {
            let new_session: IAudioSessionControl2 = new_session.cast()?;
            match unsafe { MediaDeviceSession::load(new_session, &self.device_id) } {
                Ok(session) => {
                    let tx = MediaManager::event_tx();
                    log_error!(tx.send(MediaEvent::DeviceSessionAdded {
                        device_id: self.device_id.clone(),
                        session
                    }))
                }
                Err(e) => log::error!("Failed to load session: {e:?}"),
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

    pub fn session(&self, session_id: &str) -> Option<&MediaDeviceSession> {
        self.sessions.iter().find(|s| s.id == session_id)
    }

    pub fn session_mut(&mut self, session_id: &str) -> Option<&mut MediaDeviceSession> {
        self.sessions.iter_mut().find(|s| s.id == session_id)
    }

    pub fn remove_session(&mut self, session_id: &str) {
        for session in std::mem::take(&mut self.sessions) {
            if session.id == session_id {
                session.release();
                continue;
            }
            self.sessions.push(session);
        }
    }

    pub fn release(self) {
        unsafe {
            log_error!(self
                .volume_endpoint
                .UnregisterControlChangeNotify(&self.volume_callback));
            log_error!(self
                .session_manager
                .UnregisterSessionNotification(&self.session_created_callback));
        };
        // avoid call drop and IUnknown::Release because the COM object was removed on device disconnection
        // and call Release on a unexciting object can produce a deadlock
        // std::mem::forget(self.volume_endpoint);
        // std::mem::forget(self.volume_callback);
    }
}
