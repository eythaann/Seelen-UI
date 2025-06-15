use std::ffi::OsStr;

use itertools::Itertools;
use seelen_core::system_state::{MediaPlayerOwner, MediaPlayerTimeline};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession, GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    GlobalSystemMediaTransportControlsSessionTimelineProperties, MediaPropertiesChangedEventArgs,
    PlaybackInfoChangedEventArgs, SessionsChangedEventArgs, TimelinePropertiesChangedEventArgs,
};

use crate::{
    error_handler::Result,
    log_error,
    modules::{
        media::{
            application::{MediaEvent, MediaManager},
            domain::MediaPlayer,
        },
        start::application::START_MENU_MANAGER,
    },
    trace_lock,
    utils::icon_extractor::extract_and_save_icon_umid,
    windows_api::{traits::EventRegistrationTokenExt, types::AppUserModelId, WindowsApi},
};

use super::{MediaManagerEvents, MEDIA_MANAGER};

fn timeline_from_raw(
    raw: GlobalSystemMediaTransportControlsSessionTimelineProperties,
) -> windows_core::Result<MediaPlayerTimeline> {
    // TimeSpan is in ticks of 100ns
    Ok(MediaPlayerTimeline {
        start: raw.StartTime()?.Duration * 100,
        end: raw.EndTime()?.Duration * 100,
        position: raw.Position()?.Duration * 100,
        min_seek: raw.MinSeekTime()?.Duration * 100,
        max_seek: raw.MaxSeekTime()?.Duration * 100,
        last_updated_time: raw.LastUpdatedTime()?.UniversalTime,
    })
}

impl MediaManagerEvents {
    pub(super) fn on_media_players_changed(
        session_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>,
        _args: &Option<SessionsChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session_manager) = session_manager {
            let mut current_list = {
                trace_lock!(MEDIA_MANAGER)
                    .playing()
                    .iter()
                    .filter(|s| s.removed_at.is_none())
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
            let player_id = session.SourceAppUserModelId()?;
            let tx = MediaManager::event_tx();
            let event = MediaEvent::MediaPlayerPlaybackStatusChanged {
                id: player_id.to_string(),
                playing: playback.PlaybackStatus()?
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
            };
            log_error!(tx.send(event));
        }
        Ok(())
    }

    pub(super) fn on_media_player_timeline_changed(
        session: &Option<GlobalSystemMediaTransportControlsSession>,
        _args: &Option<TimelinePropertiesChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session) = session {
            let tx = MediaManager::event_tx();
            let event = MediaEvent::MediaPlayerTimelineChanged {
                id: session.SourceAppUserModelId()?.to_string(),
                timeline: timeline_from_raw(session.GetTimelineProperties()?)?,
            };
            log_error!(tx.send(event));
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

        let timeline = session.GetTimelineProperties()?;
        let playback_info = session.GetPlaybackInfo()?;
        let status = playback_info.PlaybackStatus()?;

        let display_name = match &source_app_umid {
            AppUserModelId::Appx(umid) => WindowsApi::get_uwp_app_info(umid)?
                .DisplayInfo()?
                .DisplayName()?
                .to_string_lossy(),
            AppUserModelId::PropertyStore(umid) => {
                let start = START_MENU_MANAGER.load();
                let shortcut = start.get_by_file_umid(umid);
                match shortcut {
                    Some(shortcut) => shortcut
                        .path
                        .file_stem()
                        .unwrap_or_else(|| OsStr::new("Unknown"))
                        .to_string_lossy()
                        .to_string(),
                    None => "Unknown".to_string(),
                }
            }
        };

        // pre-extraction to avoid flickering on the ui
        extract_and_save_icon_umid(&source_app_umid);
        self.playing.insert(
            source_app_umid.to_string(),
            MediaPlayer {
                umid: source_app_umid.to_string(),
                title: properties.Title().unwrap_or_default().to_string_lossy(),
                author: properties.Artist().unwrap_or_default().to_string_lossy(),
                owner: MediaPlayerOwner { name: display_name },
                thumbnail: properties
                    .Thumbnail()
                    .ok()
                    .and_then(|stream| WindowsApi::extract_thumbnail_from_ref(stream).ok()),
                playing: status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
                timeline: timeline_from_raw(timeline)?,
                default: false,
                removed_at: None,
            },
        );

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
                session
                    .TimelinePropertiesChanged(&self.media_player_timeline_event_handler)?
                    .as_event_token(),
            ),
        );
        self.media_players
            .insert(source_app_umid.to_string(), session);
        Ok(())
    }

    pub(super) fn update_recommended_player(&mut self) {
        if let Ok(recommended) = self.get_recommended_player_id() {
            for (_id, player) in self.playing.iter_mut() {
                player.default = player.umid == recommended;
            }
        }
    }

    pub(super) fn release_pending_players(&mut self) -> Result<()> {
        let ids = self
            .playing
            .iter()
            .filter(|(_id, player)| {
                player
                    .removed_at
                    .is_some_and(|t| t.elapsed().as_millis() > 1500)
            })
            .map(|(id, _)| id.clone())
            .collect_vec();
        for id in ids {
            self.release_media_transport_session(&id)?;
        }
        Ok(())
    }

    pub(super) fn release_media_transport_session(&mut self, player_id: &str) -> Result<()> {
        if let Some(session) = self.media_players.remove(player_id) {
            if let Some((properties_token, playback_token, timeline_token)) =
                self.media_player_event_tokens.remove(player_id)
            {
                session.RemoveMediaPropertiesChanged(properties_token.value)?;
                session.RemovePlaybackInfoChanged(playback_token.value)?;
                session.RemoveTimelinePropertiesChanged(timeline_token.value)?;
            }
        }
        self.playing.remove(player_id);
        Ok(())
    }
}
