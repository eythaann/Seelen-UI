use std::ffi::OsStr;

use itertools::Itertools;
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus, MediaPropertiesChangedEventArgs,
    PlaybackInfoChangedEventArgs, SessionsChangedEventArgs,
};

use crate::{
    error_handler::Result,
    log_error,
    modules::{
        media::{
            application::{MediaEvent, MediaManager},
            domain::{MediaPlayer, MediaPlayerOwner},
        },
        start::application::START_MENU_MANAGER,
    },
    seelen_weg::icon_extractor::extract_and_save_icon_umid,
    trace_lock,
    windows_api::{traits::EventRegistrationTokenExt, types::AppUserModelId, WindowsApi},
};

use super::{MediaManagerEvents, MEDIA_MANAGER};

impl MediaManagerEvents {
    pub(super) fn on_media_player_properties_changed(
        session: &Option<GlobalSystemMediaTransportControlsSession>,
        _args: &Option<MediaPropertiesChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session) = session {
            let id = session.SourceAppUserModelId()?.to_string();
            let properties = session.TryGetMediaPropertiesAsync()?.get()?;
            let tx = MediaManager::event_tx();
            let result = tx.send(MediaEvent::MediaPlayerPropertiesChanged {
                id,
                title: properties.Title()?.to_string(),
                author: properties.Artist()?.to_string(),
                thumbnail: WindowsApi::extract_thumbnail_from_ref(properties.Thumbnail()?).ok(),
            });
            log_error!(result);
        }
        Ok(())
    }

    pub(super) fn on_media_player_playback_changed(
        session: &Option<GlobalSystemMediaTransportControlsSession>,
        _args: &Option<PlaybackInfoChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session) = session {
            let playback = session.GetPlaybackInfo()?;
            let tx = MediaManager::event_tx();
            let result = tx.send(MediaEvent::MediaPlayerPlaybackStatusChanged {
                id: session.SourceAppUserModelId()?.to_string(),
                playing: playback.PlaybackStatus()?
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
            });
            log_error!(result);
        }
        Ok(())
    }

    pub(super) fn on_media_players_changed(
        session_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>,
        _args: &Option<SessionsChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session_manager) = session_manager {
            let mut current_list = {
                trace_lock!(MEDIA_MANAGER)
                    .playing()
                    .iter()
                    .map(|session| session.umid.clone())
                    .collect_vec()
            };

            let tx = MediaManager::event_tx();
            for session in session_manager.GetSessions()? {
                let id = session.SourceAppUserModelId()?.to_string();
                if !current_list.contains(&id) {
                    let _ = tx.send(MediaEvent::MediaPlayerAdded(session));
                }
                current_list.retain(|x| *x != id);
            }

            for id in current_list {
                let _ = tx.send(MediaEvent::MediaPlayerRemoved(id));
            }
        }
        Ok(())
    }
}

impl MediaManager {
    pub fn get_media_player(
        &self,
        umid: &str,
    ) -> Option<&GlobalSystemMediaTransportControlsSession> {
        self.media_players.get(umid)
    }

    pub fn get_recommended_player_id(&self) -> Result<String> {
        Ok(self
            .media_player_manager
            .GetCurrentSession()?
            .SourceAppUserModelId()?
            .to_string_lossy())
    }

    pub(super) fn load_media_transport_session(
        &mut self,
        session: GlobalSystemMediaTransportControlsSession,
    ) -> Result<()> {
        let source_app_umid: AppUserModelId =
            session.SourceAppUserModelId()?.to_string_lossy().into();
        let properties = session.TryGetMediaPropertiesAsync()?.get()?;

        let playback_info = session.GetPlaybackInfo()?;
        let status = playback_info.PlaybackStatus()?;

        let display_name = match &source_app_umid {
            AppUserModelId::Appx(umid) => WindowsApi::get_uwp_app_info(umid)?
                .DisplayInfo()?
                .DisplayName()?
                .to_string_lossy(),
            AppUserModelId::PropertyStore(umid) => {
                let shortcut = START_MENU_MANAGER
                    .load()
                    .search_shortcut_with_same_umid(umid);
                match shortcut {
                    Some(shortcut) => shortcut
                        .file_stem()
                        .unwrap_or_else(|| OsStr::new("Unknown"))
                        .to_string_lossy()
                        .to_string(),
                    None => "Unknown".to_string(),
                }
            }
        };

        // pre-extraction to avoid flickering on the ui
        let _ = extract_and_save_icon_umid(&source_app_umid);
        self.playing.push(MediaPlayer {
            umid: source_app_umid.to_string(),
            title: properties.Title().unwrap_or_default().to_string_lossy(),
            author: properties.Artist().unwrap_or_default().to_string_lossy(),
            owner: MediaPlayerOwner { name: display_name },
            thumbnail: properties
                .Thumbnail()
                .ok()
                .and_then(|stream| WindowsApi::extract_thumbnail_from_ref(stream).ok()),
            playing: status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
            default: false,
        });

        // listen for media transport events
        self.media_player_event_tokens.insert(
            source_app_umid.to_string(),
            (
                session
                    .MediaPropertiesChanged(&self.media_player_properties_event_handler)?
                    .as_event_token(),
                session
                    .PlaybackInfoChanged(&self.media_player_playback_event_handler)?
                    .as_event_token(),
            ),
        );
        self.media_players
            .insert(source_app_umid.to_string(), session);
        Ok(())
    }

    pub(super) fn update_recommended_player(&mut self) {
        if let Ok(recommended) = self.get_recommended_player_id() {
            for player in &mut self.playing {
                player.default = player.umid == recommended;
            }
        }
    }

    pub(super) fn release_media_transport_session(&mut self, player_id: &str) -> Result<()> {
        if let Some(session) = self.media_players.remove(player_id) {
            if let Some((properties_token, playback_token)) =
                self.media_player_event_tokens.remove(player_id)
            {
                session.RemoveMediaPropertiesChanged(properties_token.value)?;
                session.RemovePlaybackInfoChanged(playback_token.value)?;
            }
        }
        self.playing.retain(|player| player.umid != player_id);
        Ok(())
    }
}
