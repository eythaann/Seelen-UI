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
        let success = session.TrySkipNextAsync()?.get()?;
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
        let success = session.TrySkipPreviousAsync()?.get()?;
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
        let success = session.TryTogglePlayPauseAsync()?.get()?;
        if !success {
            return Err("failed to toggle play".into());
        }
    }
    Ok(())
}
