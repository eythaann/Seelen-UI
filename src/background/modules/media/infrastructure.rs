use seelen_core::handlers::SeelenEvent;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Emitter;
use windows::core::GUID;

use crate::{
    error_handler::Result, modules::media::application::MEDIA_MANAGER, seelen::get_app_handle,
    trace_lock, windows_api::WindowsApi,
};

use super::domain::{Device, MediaPlayer};

fn emit_media_sessions(playing: &Vec<MediaPlayer>) {
    let app = get_app_handle();
    app.emit(SeelenEvent::MediaSessions, playing)
        .expect("failed to emit");
}

fn emit_media_devices(inputs: &Vec<Device>, outputs: &Vec<Device>) {
    let app = get_app_handle();
    app.emit(SeelenEvent::MediaInputs, inputs)
        .expect("failed to emit");
    app.emit(SeelenEvent::MediaOutputs, outputs)
        .expect("failed to emit");
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_media_events() {
    let was_registered = REGISTERED.load(Ordering::Acquire);
    if !was_registered {
        REGISTERED.store(true, Ordering::Release);
    }
    std::thread::spawn(move || {
        let mut manager = trace_lock!(MEDIA_MANAGER);
        if !was_registered {
            log::trace!("Registering media events");
            manager.on_change_devices(emit_media_devices);
            manager.on_change_players(emit_media_sessions);
        }
        emit_media_devices(manager.inputs(), manager.outputs());
        emit_media_sessions(manager.playing());
    });
}

pub fn release_media_events() {
    if REGISTERED.load(Ordering::Acquire) {
        trace_lock!(MEDIA_MANAGER).release();
    }
}

#[tauri::command(async)]
pub fn media_set_default_device(id: String, role: String) -> Result<()> {
    trace_lock!(MEDIA_MANAGER).set_default_device(&id, &role)?;
    Ok(())
}

#[tauri::command(async)]
pub fn media_next(id: String) -> Result<()> {
    if let Some(session) = trace_lock!(MEDIA_MANAGER).session_by_id(&id) {
        let success = WindowsApi::wait_for_async(session.TrySkipNextAsync()?, None)?;
        if !success {
            return Err("failed to skip next".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_prev(id: String) -> Result<()> {
    if let Some(session) = trace_lock!(MEDIA_MANAGER).session_by_id(&id) {
        let success = WindowsApi::wait_for_async(session.TrySkipPreviousAsync()?, None)?;
        if !success {
            return Err("failed to skip previous".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_toggle_play_pause(id: String) -> Result<()> {
    if let Some(session) = trace_lock!(MEDIA_MANAGER).session_by_id(&id) {
        let success = WindowsApi::wait_for_async(session.TryTogglePlayPauseAsync()?, None)?;
        if !success {
            return Err("failed to toggle play".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
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

#[tauri::command(async)]
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
