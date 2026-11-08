use std::{ffi::OsStr, sync::Arc, time::Duration};

use seelen_core::system_state::{MediaPlayer, MediaPlayerOwner, MediaPlayerTimeline};
use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession as GsmtcSession,
        GlobalSystemMediaTransportControlsSessionManager as GsmtcManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as GsmtcPlaybackStatus,
        GlobalSystemMediaTransportControlsSessionTimelineProperties as GsmtcTimeline,
        MediaPropertiesChangedEventArgs, PlaybackInfoChangedEventArgs, SessionsChangedEventArgs,
        TimelinePropertiesChangedEventArgs,
    },
};

use crate::{
    error::{Result, ResultLogExt},
    resources::icon_extractor::request_icon_extraction_from_umid,
    system_modules::{
        media_players::domain::{MediaPlayersSysEvent, MediaPlayersSystem, PlayerProperties},
        start::StartMenuManager,
    },
    utils::lock_free::SyncHashMap,
    windows_api::{WindowsApi, types::AppUserModelId},
};

/// A recently added session can fail with 0x80070015 "The device is not ready.", so reading
/// its properties is retried this many times.
const LOAD_ATTEMPTS: u32 = 15;
const LOAD_RETRY_DELAY: Duration = Duration::from_millis(10);

type EventCallback = Arc<dyn Fn(MediaPlayersSysEvent) + Send + Sync>;

pub struct WindowsMediaPlayers {
    manager: Option<GsmtcManager>,
    sessions_changed_token: Option<i64>,
    on_event: Option<EventCallback>,
    /// Loaded players, by id (umid).
    sessions: SyncHashMap<String, PlayerSession>,
}

// SAFETY: the GSMTC objects are agile WinRT objects; they are only stored to call their methods
// and to remove the event handlers (as the previous manager did, with `unsafe impl Send/Sync`).
unsafe impl Send for WindowsMediaPlayers {}
unsafe impl Sync for WindowsMediaPlayers {}

impl MediaPlayersSystem for WindowsMediaPlayers {
    fn create() -> Self {
        Self {
            manager: None,
            sessions_changed_token: None,
            on_event: None,
            sessions: SyncHashMap::new(),
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(MediaPlayersSysEvent) + Send + Sync + 'static,
    {
        let on_event: EventCallback = Arc::new(on_event);
        self.on_event = Some(on_event.clone());

        let manager = GsmtcManager::RequestAsync()?.join()?;
        let token = manager.SessionsChanged(&TypedEventHandler::new(
            move |manager: windows_core::Ref<GsmtcManager>,
                  _args: windows_core::Ref<SessionsChangedEventArgs>| {
                if let Some(manager) = manager.as_ref() {
                    on_event(MediaPlayersSysEvent::PlayersChanged {
                        ids: session_ids(manager)?,
                    });
                }
                Ok(())
            },
        ))?;
        self.sessions_changed_token = Some(token);
        self.manager = Some(manager);
        Ok(())
    }

    fn player_ids(&self) -> Result<Vec<String>> {
        Ok(session_ids(self.manager()?)?)
    }

    fn load_player(&self, id: &str) -> Result<MediaPlayer> {
        let Some(session) = self
            .manager()?
            .GetSessions()?
            .into_iter()
            .find(|s| s.SourceAppUserModelId().is_ok_and(|umid| umid == id))
        else {
            return Err(format!("Media player {id} not found").into());
        };

        let mut attempts = 0;
        let properties = loop {
            match session.TryGetMediaPropertiesAsync()?.join() {
                Ok(properties) => break properties,
                Err(err) if attempts >= LOAD_ATTEMPTS => return Err(err.into()),
                Err(_) => {
                    attempts += 1;
                    std::thread::sleep(LOAD_RETRY_DELAY);
                }
            }
        };

        let umid: AppUserModelId = id.to_owned().into();
        // pre-extraction to avoid flickering on the ui
        request_icon_extraction_from_umid(&umid);

        let player = MediaPlayer {
            umid: id.to_owned(),
            title: properties.Title().unwrap_or_default().to_string_lossy(),
            author: properties.Artist().unwrap_or_default().to_string_lossy(),
            owner: MediaPlayerOwner {
                name: display_name(&umid)?,
            },
            thumbnail: properties
                .Thumbnail()
                .ok()
                .and_then(|stream| WindowsApi::extract_thumbnail_from_ref(stream).ok()),
            playing: session.GetPlaybackInfo()?.PlaybackStatus()? == GsmtcPlaybackStatus::Playing,
            timeline: timeline_from_native(session.GetTimelineProperties()?)?,
            default: false,
        };

        let on_event = self
            .on_event
            .clone()
            .ok_or("Media players not initialized")?;
        self.sessions
            .upsert(id.to_owned(), PlayerSession::create(session, on_event)?);
        Ok(player)
    }

    fn release_player(&self, id: &str) {
        // Drop removes the event handlers
        self.sessions.remove(id);
    }

    fn recommended_player_id(&self) -> Result<String> {
        Ok(self
            .manager()?
            .GetCurrentSession()?
            .SourceAppUserModelId()?
            .to_string_lossy())
    }

    fn properties(&self, id: &str) -> Result<PlayerProperties> {
        let properties = self.session(id)?.TryGetMediaPropertiesAsync()?.join()?;
        // some players (e.g. system sounds) do not expose title, artist, or thumbnail.
        Ok(PlayerProperties {
            title: properties.Title().unwrap_or_default().to_string_lossy(),
            author: properties.Artist().unwrap_or_default().to_string_lossy(),
            thumbnail: properties
                .Thumbnail()
                .ok()
                .and_then(|s| WindowsApi::extract_thumbnail_from_ref(s).ok()),
        })
    }

    fn is_playing(&self, id: &str) -> Result<bool> {
        let status = self.session(id)?.GetPlaybackInfo()?.PlaybackStatus()?;
        Ok(status == GsmtcPlaybackStatus::Playing)
    }

    fn timeline(&self, id: &str) -> Result<MediaPlayerTimeline> {
        Ok(timeline_from_native(
            self.session(id)?.GetTimelineProperties()?,
        )?)
    }

    fn skip_next(&self, id: &str) -> Result<()> {
        self.control(id, "skip next", |s| s.TrySkipNextAsync()?.join())
    }

    fn skip_previous(&self, id: &str) -> Result<()> {
        self.control(id, "skip previous", |s| s.TrySkipPreviousAsync()?.join())
    }

    fn toggle_play_pause(&self, id: &str) -> Result<()> {
        self.control(id, "toggle play", |s| s.TryTogglePlayPauseAsync()?.join())
    }

    fn seek(&self, id: &str, position: i64) -> Result<()> {
        // Windows expects the position in 100ns ticks
        let ticks = position / 100;
        self.control(id, "seek", |s| {
            s.TryChangePlaybackPositionAsync(ticks)?.join()
        })
    }
}

impl WindowsMediaPlayers {
    fn manager(&self) -> Result<&GsmtcManager> {
        Ok(self
            .manager
            .as_ref()
            .ok_or("Media players not initialized")?)
    }

    fn session(&self, id: &str) -> Result<GsmtcSession> {
        Ok(self
            .sessions
            .get(id, |s| s.session.clone())
            .ok_or_else(|| format!("Media player {id} not loaded"))?)
    }

    /// Runs a media control. Unknown players are ignored.
    fn control(
        &self,
        id: &str,
        action: &str,
        f: impl FnOnce(&GsmtcSession) -> windows_core::Result<bool>,
    ) -> Result<()> {
        let Some(session) = self.sessions.get(id, |s| s.session.clone()) else {
            return Ok(());
        };
        if !f(&session)? {
            return Err(format!("failed to {action}").into());
        }
        Ok(())
    }
}

impl Drop for WindowsMediaPlayers {
    fn drop(&mut self) {
        self.sessions.clear();
        if let (Some(manager), Some(token)) = (&self.manager, self.sessions_changed_token.take()) {
            manager.RemoveSessionsChanged(token).log_error();
        }
    }
}

/// A loaded GSMTC session: registers its event handlers on creation and removes them on drop.
struct PlayerSession {
    session: GsmtcSession,
    properties_token: i64,
    playback_token: i64,
    timeline_token: i64,
}

impl PlayerSession {
    fn create(session: GsmtcSession, on_event: EventCallback) -> Result<Self> {
        let cb = on_event.clone();
        let properties_token = session.MediaPropertiesChanged(&TypedEventHandler::new(
            move |session: windows_core::Ref<GsmtcSession>,
                  _args: windows_core::Ref<MediaPropertiesChangedEventArgs>| {
                if let Some(session) = session.as_ref() {
                    let id = session.SourceAppUserModelId()?.to_string();
                    cb(MediaPlayersSysEvent::PropertiesChanged { id });
                }
                Ok(())
            },
        ))?;

        let cb = on_event.clone();
        let playback_token = session.PlaybackInfoChanged(&TypedEventHandler::new(
            move |session: windows_core::Ref<GsmtcSession>,
                  _args: windows_core::Ref<PlaybackInfoChangedEventArgs>| {
                if let Some(session) = session.as_ref() {
                    let id = session.SourceAppUserModelId()?.to_string();
                    cb(MediaPlayersSysEvent::PlaybackChanged { id });
                }
                Ok(())
            },
        ))?;

        let timeline_token = session.TimelinePropertiesChanged(&TypedEventHandler::new(
            move |session: windows_core::Ref<GsmtcSession>,
                  _args: windows_core::Ref<TimelinePropertiesChangedEventArgs>| {
                if let Some(session) = session.as_ref() {
                    let id = session.SourceAppUserModelId()?.to_string();
                    on_event(MediaPlayersSysEvent::TimelineChanged { id });
                }
                Ok(())
            },
        ))?;

        Ok(Self {
            session,
            properties_token,
            playback_token,
            timeline_token,
        })
    }
}

impl Drop for PlayerSession {
    fn drop(&mut self) {
        self.session
            .RemoveMediaPropertiesChanged(self.properties_token)
            .log_error();
        self.session
            .RemovePlaybackInfoChanged(self.playback_token)
            .log_error();
        self.session
            .RemoveTimelinePropertiesChanged(self.timeline_token)
            .log_error();
    }
}

fn session_ids(manager: &GsmtcManager) -> windows_core::Result<Vec<String>> {
    manager
        .GetSessions()?
        .into_iter()
        .map(|session| Ok(session.SourceAppUserModelId()?.to_string_lossy()))
        .collect()
}

/// Name of the app owning the player.
fn display_name(umid: &AppUserModelId) -> Result<String> {
    Ok(match umid {
        AppUserModelId::Appx(umid) => WindowsApi::get_uwp_app_info(umid)?
            .DisplayInfo()?
            .DisplayName()?
            .to_string_lossy(),
        AppUserModelId::PropertyStore(umid) => {
            match StartMenuManager::instance().get_by_file_umid(umid) {
                Some(shortcut) => shortcut
                    .path
                    .file_stem()
                    .unwrap_or_else(|| OsStr::new("Unknown"))
                    .to_string_lossy()
                    .to_string(),
                None => "Unknown".to_string(),
            }
        }
    })
}

fn timeline_from_native(raw: GsmtcTimeline) -> windows_core::Result<MediaPlayerTimeline> {
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
