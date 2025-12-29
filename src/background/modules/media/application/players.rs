use std::{ffi::OsStr, time::Duration};

use seelen_core::system_state::{MediaPlayerOwner, MediaPlayerTimeline};
use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
        GlobalSystemMediaTransportControlsSessionTimelineProperties,
        MediaPropertiesChangedEventArgs, PlaybackInfoChangedEventArgs, SessionsChangedEventArgs,
        TimelinePropertiesChangedEventArgs,
    },
    Win32::System::WinRT::EventRegistrationToken,
};

use crate::{
    error::{Result, ResultLogExt},
    log_error,
    modules::{
        media::{
            application::{MediaEvent, MediaManager},
            domain::MediaPlayer,
        },
        start::application::StartMenuManager,
    },
    utils::{icon_extractor::extract_and_save_icon_umid, lock_free::SyncHashMap},
    windows_api::{traits::EventRegistrationTokenExt, types::AppUserModelId, WindowsApi},
};

use super::{MediaManagerEvents, MEDIA_MANAGER};

fn timeline_from_raw(
    raw: GlobalSystemMediaTransportControlsSessionTimelineProperties,
) -> windows_core::Result<MediaPlayerTimeline> {
    // TimeSpan is in ticks of 100ns
    Ok(MediaPlayerTimeline {
        start: raw.StartTime()?.Duration.saturating_mul(100),
        end: raw.EndTime()?.Duration.saturating_mul(100),
        position: raw.Position()?.Duration.saturating_mul(100),
        min_seek: raw.MinSeekTime()?.Duration.saturating_mul(100),
        max_seek: raw.MaxSeekTime()?.Duration.saturating_mul(100),
        last_updated_time: raw.LastUpdatedTime()?.UniversalTime,
    })
}

pub struct MediaPlayersManager {
    pub playing: SyncHashMap<String, MediaPlayer>,

    manager: GlobalSystemMediaTransportControlsSessionManager,
    manager_event_handler: TypedEventHandler<
        GlobalSystemMediaTransportControlsSessionManager,
        SessionsChangedEventArgs,
    >,

    sessions: SyncHashMap<String, GlobalSystemMediaTransportControlsSession>,
    properties_event_handler: TypedEventHandler<
        GlobalSystemMediaTransportControlsSession,
        MediaPropertiesChangedEventArgs,
    >,
    playback_event_handler:
        TypedEventHandler<GlobalSystemMediaTransportControlsSession, PlaybackInfoChangedEventArgs>,
    timeline_event_handler: TypedEventHandler<
        GlobalSystemMediaTransportControlsSession,
        TimelinePropertiesChangedEventArgs,
    >,
    /// session id -> (media properties changed event, playback info changed event, timeline changed event)
    event_tokens: SyncHashMap<
        String,
        (
            EventRegistrationToken,
            EventRegistrationToken,
            EventRegistrationToken,
        ),
    >,
}

unsafe impl Send for MediaPlayersManager {}

impl MediaPlayersManager {
    pub fn new() -> Result<Self> {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?;

        Ok(Self {
            playing: SyncHashMap::new(),
            manager,
            manager_event_handler: TypedEventHandler::new(
                MediaManagerEvents::on_media_players_changed,
            ),
            sessions: SyncHashMap::new(),
            event_tokens: SyncHashMap::new(),
            properties_event_handler: TypedEventHandler::new(
                MediaManagerEvents::on_media_player_properties_changed,
            ),
            timeline_event_handler: TypedEventHandler::new(
                MediaManagerEvents::on_media_player_timeline_changed,
            ),
            playback_event_handler: TypedEventHandler::new(
                MediaManagerEvents::on_media_player_playback_changed,
            ),
        })
    }

    pub fn get_media_player(
        &self,
        umid: &str,
    ) -> Option<GlobalSystemMediaTransportControlsSession> {
        self.sessions.get(umid, |session| session.clone())
    }

    pub fn get_recommended_player_id(&self) -> Result<String> {
        Ok(self
            .manager
            .GetCurrentSession()?
            .SourceAppUserModelId()?
            .to_string_lossy())
    }

    pub unsafe fn initialize(&self) -> Result<()> {
        for session in self.manager.GetSessions()? {
            self.load_session(session)?;
        }

        self.update_recommended_player();
        self.manager.SessionsChanged(&self.manager_event_handler)?;

        Ok(())
    }

    pub fn process_event(&self, event: &MediaEvent) -> Result<()> {
        match event {
            MediaEvent::MediaPlayerAdded(session) => {
                let id = session.SourceAppUserModelId()?.to_string_lossy();
                let already_exists = self
                    .playing
                    .get(&id, |player| {
                        player.removed_at = None;
                    })
                    .is_some();

                if already_exists {
                    return Ok(());
                }

                // load_session could fail with 0x80070015 "The device is not ready."
                // when trying to load a recently added player so we retry a few times
                let mut max_attempts = 0;
                while session.TryGetMediaPropertiesAsync()?.get().is_err() && max_attempts < 15 {
                    max_attempts += 1;
                    std::thread::sleep(Duration::from_millis(10));
                }
                self.load_session(session.clone())?;
            }
            MediaEvent::MediaPlayerRemoved(id) => {
                self.playing.get(id, |player| {
                    player.removed_at = Some(std::time::Instant::now());
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(1500));
                        MediaManager::send(MediaEvent::MediaPlayerCleanRequested);
                    });
                });
            }
            MediaEvent::MediaPlayerCleanRequested => {
                self.release_pending_players()?;
            }
            MediaEvent::MediaPlayerPropertiesChanged {
                id,
                title,
                author,
                thumbnail,
            } => {
                self.playing.get(id, |player| {
                    player.title = title.clone();
                    player.author = author.clone();
                    player.thumbnail = thumbnail.clone();
                });
            }
            MediaEvent::MediaPlayerPlaybackStatusChanged { id, playing } => {
                self.playing.get(id, |player| {
                    player.playing = *playing;
                });
            }
            MediaEvent::MediaPlayerTimelineChanged { id, timeline } => {
                self.playing.get(id, |player| {
                    player.timeline = timeline.clone();
                });
            }
            _ => {}
        }
        Ok(())
    }

    pub fn update_recommended_player(&self) {
        if let Ok(recommended) = self.get_recommended_player_id() {
            self.playing.for_each(|(_, player)| {
                player.default = player.umid == recommended;
            });
        }
    }

    fn load_session(&self, session: GlobalSystemMediaTransportControlsSession) -> Result<()> {
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
                let start = StartMenuManager::instance();
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
        self.playing.upsert(
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
        self.event_tokens.upsert(
            source_app_umid.to_string(),
            (
                session
                    .MediaPropertiesChanged(&self.properties_event_handler)?
                    .as_event_token(),
                session
                    .PlaybackInfoChanged(&self.playback_event_handler)?
                    .as_event_token(),
                session
                    .TimelinePropertiesChanged(&self.timeline_event_handler)?
                    .as_event_token(),
            ),
        );
        self.sessions.upsert(source_app_umid.to_string(), session);
        Ok(())
    }

    fn release_pending_players(&self) -> Result<()> {
        let mut ids = Vec::new();
        self.playing.for_each(|(id, player)| {
            if player
                .removed_at
                .is_some_and(|t| t.elapsed().as_millis() > 1500)
            {
                ids.push(id.clone());
            }
        });
        for id in ids {
            self.release_session(&id)?;
        }
        Ok(())
    }

    fn release_session(&self, player_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.remove(player_id) {
            if let Some((properties_token, playback_token, timeline_token)) =
                self.event_tokens.remove(player_id)
            {
                session.RemoveMediaPropertiesChanged(properties_token.value)?;
                session.RemovePlaybackInfoChanged(playback_token.value)?;
                session.RemoveTimelinePropertiesChanged(timeline_token.value)?;
            }
        }
        self.playing.remove(player_id);
        Ok(())
    }

    /// Release all resources
    /// should be called on application exit
    pub fn release(&self) {
        for id in self.playing.keys() {
            self.release_session(&id).log_error();
        }
    }
}

impl MediaManagerEvents {
    pub(super) fn on_media_players_changed(
        session_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>,
        _args: &Option<SessionsChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session_manager) = session_manager {
            let mut current_list = Vec::new();
            MEDIA_MANAGER.players.playing.for_each(|(_, session)| {
                if session.removed_at.is_none() {
                    current_list.push(session.umid.clone());
                }
            });

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
