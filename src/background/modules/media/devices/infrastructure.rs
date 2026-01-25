use std::sync::Once;

use seelen_core::handlers::SeelenEvent;

use crate::{app::emit_to_webviews, error::Result, windows_api::WindowsApi};

use super::DevicesManager;

fn get_devices_manager() -> &'static DevicesManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        DevicesManager::subscribe(|_event| {
            let manager = DevicesManager::instance();
            let inputs = manager.get_inputs();
            let outputs = manager.get_outputs();

            emit_to_webviews(SeelenEvent::MediaDevices, (&inputs, &outputs));
            emit_to_webviews(SeelenEvent::MediaInputs, &inputs);
            emit_to_webviews(SeelenEvent::MediaOutputs, &outputs);
        });
    });
    DevicesManager::instance()
}

#[tauri::command(async)]
pub fn get_media_devices() -> Result<(serde_json::Value, serde_json::Value)> {
    let manager = get_devices_manager();
    let inputs = serde_json::to_value(manager.get_inputs())?;
    let outputs = serde_json::to_value(manager.get_outputs())?;
    Ok((inputs, outputs))
}

#[tauri::command(async)]
pub async fn media_set_default_device(id: String, role: String) -> Result<()> {
    get_devices_manager();
    WindowsApi::set_default_audio_device(&id, &role)?;
    Ok(())
}

#[tauri::command(async)]
pub fn media_toggle_mute(device_id: String, session_id: Option<String>) -> Result<()> {
    let manager = get_devices_manager();
    manager.toggle_mute(device_id, session_id)
}

#[tauri::command(async)]
pub fn set_volume_level(device_id: String, session_id: Option<String>, level: f32) -> Result<()> {
    let manager = get_devices_manager();
    manager.set_volume_level(device_id, session_id, level)
}
