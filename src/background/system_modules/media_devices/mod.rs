mod application;
mod domain;

use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::MediaDevice};

use crate::{app::emit_to_webviews, error::Result};

pub use self::{application::MediaDevicesManager, domain::MediaDevicesEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static MediaDevicesManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        MediaDevicesManager::subscribe(|event| match event {
            MediaDevicesEvent::Changed { inputs, outputs } => {
                emit_to_webviews(SeelenEvent::MediaDevices, (&inputs, &outputs));
                emit_to_webviews(SeelenEvent::MediaInputs, &inputs);
                emit_to_webviews(SeelenEvent::MediaOutputs, &outputs);
            }
            MediaDevicesEvent::DefaultOutputChanged => {}
        });
    });
    MediaDevicesManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_media_devices() -> Result<(Vec<MediaDevice>, Vec<MediaDevice>)> {
        let manager = get_manager();
        Ok((manager.inputs(), manager.outputs()))
    }

    pub fn media_set_default_device(id: String, role: String) -> Result<()> {
        get_manager().set_default_device(&id, &role)
    }

    pub fn media_toggle_mute(device_id: String, session_id: Option<String>) -> Result<()> {
        get_manager().toggle_mute(&device_id, session_id.as_deref())
    }

    pub fn set_volume_level(
        device_id: String,
        session_id: Option<String>,
        level: f32,
    ) -> Result<()> {
        get_manager().set_volume_level(&device_id, session_id.as_deref(), level)
    }
}
