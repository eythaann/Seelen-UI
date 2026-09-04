use std::sync::Once;

use seelen_core::handlers::SeelenEvent;

use crate::{app::emit_to_webviews, error::Result};

use super::{domain::MediaPlayer, PlayersManager};

fn get_players_manager() -> &'static PlayersManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        PlayersManager::subscribe(|_event| {
            emit_to_webviews(
                SeelenEvent::MediaSessions,
                PlayersManager::instance().get_playing_sessions(),
            );
        });
    });
    PlayersManager::instance()
}

#[tauri::command(async)]
pub fn get_media_sessions() -> Result<Vec<MediaPlayer>> {
    let manager = get_players_manager();
    Ok(manager.get_playing_sessions())
}

#[tauri::command(async)]
pub fn media_next(id: String) -> Result<()> {
    let manager = get_players_manager();
    if let Some(session) = manager.get_media_player(&id) {
        let success = session.TrySkipNextAsync()?.join()?;
        if !success {
            return Err("failed to skip next".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_prev(id: String) -> Result<()> {
    let manager = get_players_manager();
    if let Some(session) = manager.get_media_player(&id) {
        let success = session.TrySkipPreviousAsync()?.join()?;
        if !success {
            return Err("failed to skip previous".into());
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn media_toggle_play_pause(id: String) -> Result<()> {
    let manager = get_players_manager();
    if let Some(session) = manager.get_media_player(&id) {
        let success = session.TryTogglePlayPauseAsync()?.join()?;
        if !success {
            return Err("failed to toggle play".into());
        }
    }
    Ok(())
}

/// `position` is the target playback position in nanoseconds, matching the units
/// used by `MediaPlayerTimeline` (start/end/position/min_seek/max_seek). Windows
/// itself expects the position in 100ns ticks, so it's converted before the call.
#[tauri::command(async)]
pub fn media_seek(id: String, position: i64) -> Result<()> {
    let manager = get_players_manager();
    if let Some(session) = manager.get_media_player(&id) {
        let timeline = session.GetTimelineProperties()?;
        let mut min_seek = timeline.MinSeekTime()?.Duration.saturating_mul(100);
        let mut max_seek = timeline.MaxSeekTime()?.Duration.saturating_mul(100);
        // Not every app populates Min/MaxSeekTime, in that case both default to 0,
        // which would force every seek to position 0. Fall back to the track's
        // start/end range instead.
        if max_seek <= min_seek {
            min_seek = timeline.StartTime()?.Duration.saturating_mul(100);
            max_seek = timeline.EndTime()?.Duration.saturating_mul(100);
        }
        // Guard against apps reporting an inverted or degenerate range, which
        // would make `clamp` panic (it requires min <= max).
        if max_seek < min_seek {
            max_seek = min_seek;
        }
        let clamped = position.clamp(min_seek, max_seek);

        let ticks = clamped / 100;
        let success = session.TryChangePlaybackPositionAsync(ticks)?.join()?;
        if !success {
            return Err("failed to seek".into());
        }
    }
    Ok(())
}
