use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;
use windows::core::GUID;

use crate::{
    error_handler::Result,
    modules::media::application::MEDIA_MANAGER,
    seelen::get_app_handle,
    windows_api::{Com, WindowsApi},
};

pub fn emit_media_sessions() {
    std::thread::spawn(|| -> Result<()> {
        let manager = MEDIA_MANAGER.lock();
        let sessions = tauri::async_runtime::block_on(manager.request_media_sessions())?;
        let handle = get_app_handle();
        handle.emit("media-sessions", &sessions)?;
        Ok(())
    });
}

pub fn emit_media_volume(volume: f32, muted: bool) {
    let handle = get_app_handle();
    handle.emit("media-volume", volume).expect("failed to emit");
    handle.emit("media-muted", muted).expect("failed to emit");
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_media_events() {
    if !REGISTERED.load(Ordering::Acquire) {
        let manager = MEDIA_MANAGER.lock();
        manager.listen_transport_controls_events(emit_media_sessions);
        manager.listen_media_volume_events(emit_media_volume);
        REGISTERED.store(true, Ordering::Release);
    }

    emit_media_sessions();
    Com::run_with_context(|| {
        let audio_endpoint = WindowsApi::get_default_audio_endpoint()?;
        unsafe {
            emit_media_volume(
                audio_endpoint.GetMasterVolumeLevelScalar()?,
                audio_endpoint.GetMute()?.as_bool(),
            )
        };
        Ok(())
    })
    .expect("failed to register media events");
}

#[tauri::command]
pub fn media_next(id: String) -> Result<()> {
    if let Some(session) = MEDIA_MANAGER.lock().session_by_id(&id)? {
        let success = tauri::async_runtime::block_on(session.TrySkipNextAsync()?)?;
        if !success {
            return Err("failed to skip next".into());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn media_prev(id: String) -> Result<()> {
    if let Some(session) = MEDIA_MANAGER.lock().session_by_id(&id)? {
        let success = tauri::async_runtime::block_on(session.TrySkipPreviousAsync()?)?;
        if !success {
            return Err("failed to skip previous".into());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn media_toggle_play_pause(id: String) -> Result<()> {
    if let Some(session) = MEDIA_MANAGER.lock().session_by_id(&id)? {
        let success = tauri::async_runtime::block_on(session.TryTogglePlayPauseAsync()?)?;
        if !success {
            return Err("failed to toggle play".into());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn media_toggle_mute() -> Result<()> {
    unsafe {
        let endpoint = WindowsApi::get_default_audio_endpoint()?;
        let muted = endpoint.GetMute()?.as_bool();
        endpoint.SetMute(!muted, &GUID::zeroed())?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_volume_level(level: f32) -> Result<()> {
    unsafe {
        let endpoint = WindowsApi::get_default_audio_endpoint()?;
        endpoint.SetMasterVolumeLevelScalar(level, &GUID::zeroed())?;
    }
    Ok(())
}
