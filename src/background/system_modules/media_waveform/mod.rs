mod application;
mod domain;

use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::AudioWaveform};

use crate::{app::emit_to_webviews, error::Result};

use self::{application::WaveformManager, domain::WaveformEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static WaveformManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        WaveformManager::subscribe(|event| match event {
            WaveformEvent::Tick => {
                emit_to_webviews(
                    SeelenEvent::MediaWaveform,
                    WaveformManager::instance().latest(),
                );
            }
        });
    });
    WaveformManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_media_waveform() -> Result<AudioWaveform> {
        Ok(get_manager().latest())
    }
}
