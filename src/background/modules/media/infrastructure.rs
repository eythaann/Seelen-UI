use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;
use windows::core::GUID;

use crate::{
    error_handler::Result, modules::media::application::MEDIA_MANAGER, seelen::get_app_handle,
    trace_lock,
};

use super::domain::{Device, MediaPlayer};

fn emit_media_sessions(playing: &Vec<MediaPlayer>) {
    let app = get_app_handle();
    app.emit("media-sessions", playing).expect("failed to emit");
}

fn emit_media_devices(inputs: &Vec<Device>, outputs: &Vec<Device>) {
    let app = get_app_handle();
    app.emit("media-inputs", inputs).expect("failed to emit");
    app.emit("media-outputs", outputs).expect("failed to emit");
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_media_events() {
    std::thread::spawn(|| {
        if !REGISTERED.load(Ordering::Acquire) {
            let mut manager = MEDIA_MANAGER.lock();
            manager.on_change_devices(emit_media_devices);
            manager.on_change_players(emit_media_sessions);
            REGISTERED.store(true, Ordering::Release);
        }

        let media_manager = MEDIA_MANAGER.lock();
        emit_media_devices(media_manager.inputs(), media_manager.outputs());
        emit_media_sessions(media_manager.playing());
    });
}

pub fn release_media_events() {
    if REGISTERED.load(Ordering::Acquire) {
        MEDIA_MANAGER.lock().release();
    }
}

#[tauri::command]
pub fn media_set_default_device(id: String, role: String) -> Result<()> {
    MEDIA_MANAGER.lock().set_default_device(&id, &role)?;
    Ok(())
}

#[tauri::command]
pub fn media_next(id: String) -> Result<()> {
    if let Some(session) = MEDIA_MANAGER.lock().session_by_id(&id) {
        let success = tauri::async_runtime::block_on(session.TrySkipNextAsync()?)?;
        if !success {
            return Err("failed to skip next".into());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn media_prev(id: String) -> Result<()> {
    if let Some(session) = MEDIA_MANAGER.lock().session_by_id(&id) {
        let success = tauri::async_runtime::block_on(session.TrySkipPreviousAsync()?)?;
        if !success {
            return Err("failed to skip previous".into());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn media_toggle_play_pause(id: String) -> Result<()> {
    if let Some(session) = MEDIA_MANAGER.lock().session_by_id(&id) {
        let success = tauri::async_runtime::block_on(session.TryTogglePlayPauseAsync()?)?;
        if !success {
            return Err("failed to toggle play".into());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn media_toggle_mute(id: String, _session_id: Option<String>) -> Result<()> {
    let manager = trace_lock!(MEDIA_MANAGER);
    let endpoints = manager.devices_audio_endpoint();
    if let Some((endpoint, _)) = endpoints.get(&id) {
        unsafe {
            endpoint.SetMute(!endpoint.GetMute()?.as_bool(), &GUID::zeroed())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn set_volume_level(id: String, _session_id: Option<String>, level: f32) -> Result<()> {
    let manager = trace_lock!(MEDIA_MANAGER);
    let endpoints = manager.devices_audio_endpoint();
    if let Some((endpoint, _)) = endpoints.get(&id) {
        unsafe {
            endpoint.SetMasterVolumeLevelScalar(level, &GUID::zeroed())?;
        }
    }
    Ok(())
}
