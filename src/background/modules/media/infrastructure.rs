use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;
use windows::{core::GUID, Win32::Media::Audio::ISimpleAudioVolume};

use crate::{
    error_handler::Result, log_error, modules::media::application::MEDIA_MANAGER,
    seelen::get_app_handle, trace_lock,
};

use super::{
    application::{MediaEvent, MediaManager},
    domain::MediaPlayer,
};

pub fn register_media_events() {
    std::thread::spawn(|| unsafe {
        log_error!(trace_lock!(MEDIA_MANAGER).initialize());

        MediaManager::subscribe(|event| match event {
            MediaEvent::MediaPlayerAdded(_)
            | MediaEvent::MediaPlayerRemoved(_)
            | MediaEvent::MediaPlayerCleanRequested
            | MediaEvent::MediaPlayerPropertiesChanged { .. }
            | MediaEvent::MediaPlayerPlaybackStatusChanged { .. } => {
                let manager = trace_lock!(MEDIA_MANAGER);
                log_error!(get_app_handle().emit(SeelenEvent::MediaSessions, manager.playing()));
            }
            _ => {
                let manager = trace_lock!(MEDIA_MANAGER);
                let app = get_app_handle();
                log_error!(app.emit(SeelenEvent::MediaInputs, manager.inputs()));
                log_error!(app.emit(SeelenEvent::MediaOutputs, manager.outputs()));
            }
        });
    });
}

pub fn release_media_events() {
    trace_lock!(MEDIA_MANAGER).release();
}

#[tauri::command(async)]
pub fn get_media_devices() -> Result<(serde_json::Value, serde_json::Value)> {
    let manager = trace_lock!(MEDIA_MANAGER);
    let inputs = serde_json::to_value(manager.inputs())?;
    let outputs = serde_json::to_value(manager.outputs())?;
    Ok((inputs, outputs))
}

#[tauri::command(async)]
pub fn get_media_sessions() -> Result<Vec<MediaPlayer>> {
    let manager = trace_lock!(MEDIA_MANAGER);
    Ok(manager.playing().into_iter().cloned().collect())
}

#[tauri::command(async)]
pub fn media_set_default_device(id: String, role: String) -> Result<()> {
    MediaManager::set_default_device(&id, &role)
}

#[tauri::command(async)]
pub fn media_next(id: String) -> Result<()> {
    if let Some(session) = trace_lock!(MEDIA_MANAGER).get_media_player(&id) {
        let success = session.TrySkipNextAsync()?.get()?;
        if !success {
            return Err("failed to skip next".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_prev(id: String) -> Result<()> {
    if let Some(session) = trace_lock!(MEDIA_MANAGER).get_media_player(&id) {
        let success = session.TrySkipPreviousAsync()?.get()?;
        if !success {
            return Err("failed to skip previous".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_toggle_play_pause(id: String) -> Result<()> {
    if let Some(session) = trace_lock!(MEDIA_MANAGER).get_media_player(&id) {
        let success = session.TryTogglePlayPauseAsync()?.get()?;
        if !success {
            return Err("failed to toggle play".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_toggle_mute(device_id: String, session_id: Option<String>) -> Result<()> {
    let manager = trace_lock!(MEDIA_MANAGER);
    if let Some(device) = manager.device(&device_id) {
        if session_id.is_none() {
            unsafe {
                device.volume_endpoint.SetMute(
                    !device.volume_endpoint.GetMute()?.as_bool(),
                    &GUID::zeroed(),
                )?;
            }
            return Ok(());
        }

        if let Some(session) = device.session(&session_id.unwrap()) {
            unsafe {
                use windows_core::Interface;
                let volume: ISimpleAudioVolume = session.controls.cast()?;
                volume.SetMute(!volume.GetMute()?.as_bool(), &GUID::zeroed())?;
            }
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn set_volume_level(
    device_id: String,
    session_id: Option<String>,
    mut level: f32,
) -> Result<()> {
    let manager = trace_lock!(MEDIA_MANAGER);
    level = level.clamp(0.0, 1.0); // ensure valid value

    if let Some(device) = manager.device(&device_id) {
        if session_id.is_none() {
            unsafe {
                device
                    .volume_endpoint
                    .SetMasterVolumeLevelScalar(level, &GUID::zeroed())?;
            }
            return Ok(());
        }

        if let Some(session) = device.session(&session_id.unwrap()) {
            unsafe {
                use windows_core::Interface;
                let volume: ISimpleAudioVolume = session.controls.cast()?;
                volume.SetMasterVolume(level, &GUID::zeroed())?;
            }
        }
    }
    Ok(())
}
