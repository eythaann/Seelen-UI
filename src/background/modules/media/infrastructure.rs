use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;
use windows::{core::GUID, Win32::Media::Audio::ISimpleAudioVolume};

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    modules::media::{application::MEDIA_MANAGER, domain::MediaDevice},
    windows_api::WindowsApi,
};

use super::{
    application::{MediaEvent, MediaManager},
    domain::MediaPlayer,
};

pub fn register_media_events() {
    std::thread::spawn(|| unsafe {
        log_error!(MEDIA_MANAGER.initialize());

        MediaManager::subscribe(|event| match event {
            MediaEvent::MediaPlayerAdded(_)
            | MediaEvent::MediaPlayerRemoved(_)
            | MediaEvent::MediaPlayerCleanRequested
            | MediaEvent::MediaPlayerPropertiesChanged { .. }
            | MediaEvent::MediaPlayerPlaybackStatusChanged { .. }
            | MediaEvent::MediaPlayerTimelineChanged { .. } => {
                log_error!(get_app_handle().emit(
                    SeelenEvent::MediaSessions,
                    MEDIA_MANAGER.players.playing.values()
                ));
            }
            MediaEvent::DeviceAdded(_)
            | MediaEvent::DeviceRemoved(_)
            | MediaEvent::DefaultDeviceChanged { .. }
            | MediaEvent::DeviceVolumeChanged { .. }
            | MediaEvent::DeviceSessionAdded { .. }
            | MediaEvent::DeviceSessionRemoved { .. }
            | MediaEvent::DeviceSessionVolumeChanged { .. } => {
                let app = get_app_handle();
                log_error!(app.emit(SeelenEvent::MediaInputs, MEDIA_MANAGER.inputs.values()));
                log_error!(app.emit(SeelenEvent::MediaOutputs, MEDIA_MANAGER.outputs.values()));
            }
        });
    });
}

pub fn release_media_events() {
    MEDIA_MANAGER.release();
}

#[tauri::command(async)]
pub fn get_media_devices() -> Result<(serde_json::Value, serde_json::Value)> {
    let inputs = serde_json::to_value(MEDIA_MANAGER.inputs.values())?;
    let outputs = serde_json::to_value(MEDIA_MANAGER.outputs.values())?;
    Ok((inputs, outputs))
}

#[tauri::command(async)]
pub fn get_media_sessions() -> Result<Vec<MediaPlayer>> {
    Ok(MEDIA_MANAGER.players.playing.values())
}

#[tauri::command(async)]
pub async fn media_set_default_device(id: String, role: String) -> Result<()> {
    WindowsApi::set_default_audio_device(&id, &role)?;
    /* let program = std::env::current_exe()?;
    get_app_handle()
        .shell()
        .command(program)
        .args(["win32", "set-default-audio-device", &id, &role])
        .status()
        .await?; */
    Ok(())
}

#[tauri::command(async)]
pub fn media_next(id: String) -> Result<()> {
    if let Some(session) = MEDIA_MANAGER.get_media_player(&id) {
        let success = session.TrySkipNextAsync()?.get()?;
        if !success {
            return Err("failed to skip next".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_prev(id: String) -> Result<()> {
    if let Some(session) = MEDIA_MANAGER.get_media_player(&id) {
        let success = session.TrySkipPreviousAsync()?.get()?;
        if !success {
            return Err("failed to skip previous".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_toggle_play_pause(id: String) -> Result<()> {
    if let Some(session) = MEDIA_MANAGER.get_media_player(&id) {
        let success = session.TryTogglePlayPauseAsync()?.get()?;
        if !success {
            return Err("failed to toggle play".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_toggle_mute(device_id: String, session_id: Option<String>) -> Result<()> {
    let cb = |device: &mut MediaDevice| {
        match &session_id {
            Some(session_id) => {
                if let Some(session) = device.session(session_id) {
                    unsafe {
                        use windows_core::Interface;
                        let volume: ISimpleAudioVolume = session.controls.cast()?;
                        volume.SetMute(!volume.GetMute()?.as_bool(), &GUID::zeroed())?;
                    }
                }
            }
            None => unsafe {
                device.volume_endpoint.SetMute(
                    !device.volume_endpoint.GetMute()?.as_bool(),
                    &GUID::zeroed(),
                )?;
            },
        }
        Ok(())
    };

    if let Some(result) = MEDIA_MANAGER.inputs.get(&device_id, cb) {
        return result;
    }
    if let Some(result) = MEDIA_MANAGER.outputs.get(&device_id, cb) {
        return result;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn set_volume_level(
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
                        use windows_core::Interface;
                        let volume: ISimpleAudioVolume = session.controls.cast()?;
                        volume.SetMasterVolume(level, &GUID::zeroed())?;
                    }
                }
            }
            None => unsafe {
                device
                    .volume_endpoint
                    .SetMasterVolumeLevelScalar(level, &GUID::zeroed())?;
            },
        }
        Ok(())
    };

    if let Some(result) = MEDIA_MANAGER.inputs.get(&device_id, cb) {
        return result;
    }
    if let Some(result) = MEDIA_MANAGER.outputs.get(&device_id, cb) {
        return result;
    }
    Ok(())
}
