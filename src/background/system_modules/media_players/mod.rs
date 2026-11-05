mod application;
mod domain;

use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::MediaPlayer};

use crate::{app::emit_to_webviews, error::Result};

use self::{application::MediaPlayersManager, domain::MediaPlayersEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static MediaPlayersManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        MediaPlayersManager::subscribe(|event| match event {
            MediaPlayersEvent::Changed => {
                emit_to_webviews(
                    SeelenEvent::MediaSessions,
                    MediaPlayersManager::instance().players(),
                );
            }
        });
    });
    MediaPlayersManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_media_sessions() -> Vec<MediaPlayer> {
        get_manager().players()
    }

    pub fn media_next(id: String) -> Result<()> {
        get_manager().skip_next(&id)
    }

    pub fn media_prev(id: String) -> Result<()> {
        get_manager().skip_previous(&id)
    }

    pub fn media_toggle_play_pause(id: String) -> Result<()> {
        get_manager().toggle_play_pause(&id)
    }

    /// `position` is the target playback position in nanoseconds, matching the units
    /// used by `MediaPlayerTimeline` (start/end/position/min_seek/max_seek).
    pub fn media_seek(id: String, position: i64) -> Result<()> {
        get_manager().seek(&id, position)
    }
}
