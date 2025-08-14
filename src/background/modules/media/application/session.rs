use windows::Win32::Media::Audio::{
    IAudioSessionControl2, IAudioSessionEvents, IAudioSessionEvents_Impl, ISimpleAudioVolume,
};

use crate::{
    error::Result,
    log_error,
    modules::media::{application::MediaEvent, domain::MediaDeviceSession},
    windows_api::process::Process,
};

use windows_core::Interface;

use super::MediaManager;

#[windows::core::implement(IAudioSessionEvents)]
struct MediaSessionEventHandler {
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
        let tx = MediaManager::event_tx();
        let result = tx.send(MediaEvent::DeviceSessionRemoved {
            device_id: self.device_id.clone(),
            session_id: self.session_id.clone(),
        });
        log_error!(result);
        Ok(())
    }

    fn OnSimpleVolumeChanged(
        &self,
        new_volume: f32,
        new_mute: windows::Win32::Foundation::BOOL,
        _event_context: *const windows::core::GUID,
    ) -> windows::core::Result<()> {
        let tx = MediaManager::event_tx();
        let result = tx.send(MediaEvent::DeviceSessionVolumeChanged {
            device_id: self.device_id.clone(),
            session_id: self.session_id.clone(),
            volume: new_volume,
            muted: new_mute.as_bool(),
        });
        log_error!(result);
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

    pub fn release(self) {
        unsafe {
            log_error!(self
                .controls
                .UnregisterAudioSessionNotification(&self.events_callback))
        }
    }
}
