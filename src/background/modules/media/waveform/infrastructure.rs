use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::AudioWaveform};

use crate::{app::emit_to_webviews, error::Result};

use super::WaveformManager;

fn get_waveform_manager() -> &'static WaveformManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        WaveformManager::subscribe(|_event| {
            emit_to_webviews(
                SeelenEvent::MediaWaveform,
                WaveformManager::instance().get_latest(),
            );
        });
    });
    WaveformManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_media_waveform() -> Result<AudioWaveform> {
        Ok(get_waveform_manager().get_latest())
    }
}
