use std::{ffi::OsStr, path::PathBuf, sync::LazyLock, time::Duration};

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
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    modules::start::application::StartMenuManager,
    utils::{icon_extractor::request_icon_extraction_from_umid, lock_free::SyncHashMap},
    windows_api::{types::AppUserModelId, WindowsApi},
};

use super::domain::{MediaPlayer, MediaPlayerSession};

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

#[derive(Debug, Clone)]
pub enum PlayersEvent {
    PlayerAdded(GlobalSystemMediaTransportControlsSession),
    PlayerRemoved(String),
    CleanRequested,
    PropertiesChanged {
        id: String,
        title: String,
        author: String,
        thumbnail: Option<PathBuf>,
    },
    PlaybackStatusChanged {
        id: String,
        playing: bool,
    },
    TimelineChanged {
        id: String,
        timeline: MediaPlayerTimeline,
    },
}

unsafe impl Send for PlayersEvent {}

event_manager!(PlayersManager, PlayersEvent);

pub struct PlayersManager {
    playing: SyncHashMap<String, MediaPlayer>,
    sessions: SyncHashMap<String, MediaPlayerSession>,
    manager: GlobalSystemMediaTransportControlsSessionManager,
}

unsafe impl Send for PlayersManager {}
unsafe impl Sync for PlayersManager {}

impl PlayersManager {
    fn new() -> Result<Self> {
        let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.get()?;

        Ok(Self {
            playing: SyncHashMap::new(),
            sessions: SyncHashMap::new(),
            manager,
        })
    }

    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<PlayersManager> = LazyLock::new(|| {
            let mut manager = PlayersManager::new().expect("Failed to create players manager");
            unsafe { manager.init().log_error() };
            manager
        });
        &MANAGER
    }

    unsafe fn init(&mut self) -> Result<()> {
        for session in self.manager.GetSessions()? {
            self.load_session(session)?;
        }
        self.update_recommended_player();

        self.manager
            .SessionsChanged(&TypedEventHandler::new(Self::on_media_players_changed))?;

        let eid = Self::subscribe(|event| {
            PlayersManager::instance().process_event(&event).log_error();
            PlayersManager::instance().update_recommended_player();
        });
        Self::set_event_handler_priority(&eid, 1);

        Ok(())
    }

    pub fn get_media_player(
        &self,
        umid: &str,
    ) -> Option<GlobalSystemMediaTransportControlsSession> {
        self.sessions.get(umid, |session| session.session.clone())
    }

    pub fn get_recommended_player_id(&self) -> Result<String> {
        Ok(self
            .manager
            .GetCurrentSession()?
            .SourceAppUserModelId()?
            .to_string_lossy())
    }

    pub fn get_playing_sessions(&self) -> Vec<MediaPlayer> {
        self.playing.values()
    }

    fn process_event(&self, event: &PlayersEvent) -> Result<()> {
        match event {
            PlayersEvent::PlayerAdded(session) => {
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
            PlayersEvent::PlayerRemoved(id) => {
                self.playing.get(id, |player| {
                    player.removed_at = Some(std::time::Instant::now());
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(1000));
                        PlayersManager::send(PlayersEvent::CleanRequested);
                    });
                });
            }
            PlayersEvent::CleanRequested => {
                self.release_pending_players()?;
            }
            PlayersEvent::PropertiesChanged {
                id,
                title,
                author,
                thumbnail,
            } => {
                self.playing.get(id, |player| {
                    player.base.title = title.clone();
                    player.base.author = author.clone();
                    player.base.thumbnail = thumbnail.clone();
                });
            }
            PlayersEvent::PlaybackStatusChanged { id, playing } => {
                self.playing.get(id, |player| {
                    player.base.playing = *playing;
                });
            }
            PlayersEvent::TimelineChanged { id, timeline } => {
                self.playing.get(id, |player| {
                    player.base.timeline = timeline.clone();
                });
            }
        }
        Ok(())
    }

    fn update_recommended_player(&self) {
        if let Ok(recommended) = self.get_recommended_player_id() {
            self.playing.for_each(|(_, player)| {
                player.base.default = player.base.umid == recommended;
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
        request_icon_extraction_from_umid(&source_app_umid);
        self.playing.upsert(
            source_app_umid.to_string(),
            MediaPlayer {
                base: seelen_core::system_state::MediaPlayer {
                    umid: source_app_umid.to_string(),
                    title: properties.Title().unwrap_or_default().to_string_lossy(),
                    author: properties.Artist().unwrap_or_default().to_string_lossy(),
                    owner: MediaPlayerOwner { name: display_name },
                    thumbnail: properties
                        .Thumbnail()
                        .ok()
                        .and_then(|stream| WindowsApi::extract_thumbnail_from_ref(stream).ok()),
                    playing: status
                        == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
                    timeline: timeline_from_raw(timeline)?,
                    default: false,
                },
                removed_at: None,
            },
        );

        let wrapped_session = MediaPlayerSession::create(
            session,
            &TypedEventHandler::new(Self::on_media_player_properties_changed),
            &TypedEventHandler::new(Self::on_media_player_playback_changed),
            &TypedEventHandler::new(Self::on_media_player_timeline_changed),
        )?;

        self.sessions
            .upsert(source_app_umid.to_string(), wrapped_session);
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
        // Remove session from map - Drop trait will handle event unregistration
        self.sessions.remove(player_id);
        self.playing.remove(player_id);
        Ok(())
    }

    fn on_media_players_changed(
        session_manager: &Option<GlobalSystemMediaTransportControlsSessionManager>,
        _args: &Option<SessionsChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session_manager) = session_manager {
            let mut current_list = Vec::new();
            PlayersManager::instance().playing.for_each(|(_, session)| {
                if session.removed_at.is_none() {
                    current_list.push(session.base.umid.clone());
                }
            });

            let tx = PlayersManager::event_tx();
            for session in session_manager.GetSessions()? {
                let id = session.SourceAppUserModelId()?.to_string();
                if !current_list.contains(&id) {
                    let _ = tx.send(PlayersEvent::PlayerAdded(session));
                }
                current_list.retain(|x| *x != id);
            }

            for id in current_list {
                let _ = tx.send(PlayersEvent::PlayerRemoved(id));
            }
        }
        Ok(())
    }

    fn on_media_player_properties_changed(
        session: &Option<GlobalSystemMediaTransportControlsSession>,
        _args: &Option<MediaPropertiesChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session) = session {
            let id = session.SourceAppUserModelId()?.to_string();
            let properties = session.TryGetMediaPropertiesAsync()?.get()?;
            let tx = PlayersManager::event_tx();
            let result = tx.send(PlayersEvent::PropertiesChanged {
                id,
                title: properties.Title()?.to_string(),
                author: properties.Artist()?.to_string(),
                thumbnail: WindowsApi::extract_thumbnail_from_ref(properties.Thumbnail()?).ok(),
            });
            result.log_error();
        }
        Ok(())
    }

    fn on_media_player_playback_changed(
        session: &Option<GlobalSystemMediaTransportControlsSession>,
        _args: &Option<PlaybackInfoChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session) = session {
            let playback = session.GetPlaybackInfo()?;
            let player_id = session.SourceAppUserModelId()?;
            let tx = PlayersManager::event_tx();
            let event = PlayersEvent::PlaybackStatusChanged {
                id: player_id.to_string(),
                playing: playback.PlaybackStatus()?
                    == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing,
            };
            tx.send(event).log_error();
        }
        Ok(())
    }

    fn on_media_player_timeline_changed(
        session: &Option<GlobalSystemMediaTransportControlsSession>,
        _args: &Option<TimelinePropertiesChangedEventArgs>,
    ) -> windows_core::Result<()> {
        if let Some(session) = session {
            let tx = PlayersManager::event_tx();
            let event = PlayersEvent::TimelineChanged {
                id: session.SourceAppUserModelId()?.to_string(),
                timeline: timeline_from_raw(session.GetTimelineProperties()?)?,
            };
            tx.send(event).log_error();
        }
        Ok(())
    }
}
