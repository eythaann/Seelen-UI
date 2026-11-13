#[cfg(windows)]
mod windows;

use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use seelen_core::system_state::{MediaPlayer, MediaTransportAction};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::SyncHashMap,
};

use super::domain::{MediaPlayersEvent, MediaPlayersSysEvent, MediaPlayersSystem, seek_range};

#[cfg(windows)]
type Platform = windows::WindowsMediaPlayers;

/// Grace period a player must have been absent before it is released: some players vanish
/// and come back on track changes, this avoids flickering on the UI.
const REMOVAL_GRACE: Duration = Duration::from_millis(1500);
/// Delay before releasing the players pending removal.
/// Must be strictly greater than REMOVAL_GRACE so elapsed > threshold when processed.
const REMOVAL_SCHEDULE: Duration = Duration::from_millis(2000);
/// Delay before retrying the thumbnail fetch after a missing thumbnail.
const THUMBNAIL_RETRY_DELAY: Duration = Duration::from_millis(300);

pub struct MediaPlayersManager {
    platform: Platform,
    players: SyncHashMap<String, PlayerEntry>,
}

event_manager!(MediaPlayersManager, MediaPlayersEvent);

impl MediaPlayersManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<MediaPlayersManager> = LazyLock::new(|| {
            let mut m = MediaPlayersManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            players: SyncHashMap::new(),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.platform.init(Self::on_platform_event)?;
        for id in self.platform.player_ids()? {
            self.load_player(&id).log_error();
        }
        self.update_recommended_player();
        Ok(())
    }

    fn on_platform_event(event: MediaPlayersSysEvent) {
        let manager = Self::instance();
        match event {
            MediaPlayersSysEvent::PlayersChanged { ids } => manager.sync_players(ids),
            MediaPlayersSysEvent::PropertiesChanged { id } => manager.refresh_properties(&id),
            MediaPlayersSysEvent::PlaybackChanged { id } => {
                if let Some(playing) = manager.platform.is_playing(&id).ok_logged() {
                    manager.update_player(&id, |player| player.playing = playing);
                }
                if let Some(controls) = manager.platform.controls(&id).ok_logged() {
                    manager.update_player(&id, |player| player.controls = controls);
                }
                manager.update_recommended_player();
            }
            MediaPlayersSysEvent::TimelineChanged { id } => {
                if let Some(timeline) = manager.platform.timeline(&id).ok_logged() {
                    manager.update_player(&id, |player| player.timeline = timeline);
                }
            }
        }
        Self::send(MediaPlayersEvent::Changed);
    }

    /// Players not pending removal.
    pub fn players(&self) -> Vec<MediaPlayer> {
        let mut players = Vec::new();
        self.players.for_each(|(_, entry)| {
            if entry.removed_at.is_none() {
                players.push(entry.player.clone());
            }
        });
        players
    }

    fn update_player(&self, id: &str, f: impl FnOnce(&mut MediaPlayer)) {
        self.players.get(id, |entry| f(&mut entry.player));
    }

    /// Adds the new players and schedules the removal of the missing ones.
    fn sync_players(&self, ids: Vec<String>) {
        for id in &ids {
            let known = self
                .players
                .get(id, |entry| entry.removed_at = None)
                .is_some();
            if !known {
                self.load_player(id).log_error();
            }
        }

        let mut schedule_clean = false;
        self.players.for_each(|(id, entry)| {
            // Only start the removal timer if not already pending removal.
            // Avoids resetting the timer on repeated notifications.
            if !ids.contains(id) && entry.removed_at.is_none() {
                entry.removed_at = Some(Instant::now());
                schedule_clean = true;
            }
        });
        if schedule_clean {
            Self::schedule_release();
        }
        self.update_recommended_player();
    }

    fn load_player(&self, id: &str) -> Result<()> {
        let player = self.platform.load_player(id)?;
        self.players.upsert(
            id.to_owned(),
            PlayerEntry {
                player,
                removed_at: None,
            },
        );
        Ok(())
    }

    fn schedule_release() {
        std::thread::spawn(|| {
            std::thread::sleep(REMOVAL_SCHEDULE);
            Self::instance().release_pending_players();
            Self::send(MediaPlayersEvent::Changed);
        });
    }

    fn release_pending_players(&self) {
        let mut ids_to_release = Vec::new();
        let mut needs_reschedule = false;
        self.players.for_each(|(id, entry)| {
            if let Some(removed_at) = entry.removed_at {
                if removed_at.elapsed() > REMOVAL_GRACE {
                    ids_to_release.push(id.clone());
                } else {
                    // Timer fired early (the player was removed again meanwhile);
                    // reschedule so this player is eventually cleaned up.
                    needs_reschedule = true;
                }
            }
        });
        for id in ids_to_release {
            self.platform.release_player(&id);
            self.players.remove(&id);
        }
        if needs_reschedule {
            Self::schedule_release();
        }
        self.update_recommended_player();
    }

    fn refresh_properties(&self, id: &str) {
        let Some(properties) = self.platform.properties(id).ok_logged() else {
            return;
        };
        let missing_thumbnail = properties.thumbnail.is_none();
        self.update_player(id, |player| {
            player.title = properties.title;
            player.author = properties.author;
            player.thumbnail = properties.thumbnail;
        });

        // some players set the thumbnail a bit after the other properties
        if missing_thumbnail {
            let id = id.to_owned();
            std::thread::spawn(move || {
                std::thread::sleep(THUMBNAIL_RETRY_DELAY);
                Self::instance().retry_thumbnail(&id);
            });
        }
    }

    fn retry_thumbnail(&self, id: &str) {
        let still_missing = self
            .players
            .get(id, |entry| entry.player.thumbnail.is_none())
            .unwrap_or(false);
        if !still_missing {
            return;
        }
        let thumbnail = self.platform.properties(id).ok().and_then(|p| p.thumbnail);
        if let Some(thumbnail) = thumbnail {
            self.update_player(id, |player| player.thumbnail = Some(thumbnail));
            Self::send(MediaPlayersEvent::Changed);
        }
    }

    fn update_recommended_player(&self) {
        if let Ok(recommended) = self.platform.recommended_player_id() {
            self.players.for_each(|(_, entry)| {
                entry.player.default = entry.player.umid == recommended;
            });
        }
    }

    pub fn send_app_shortcut(
        &self,
        hwnd: isize,
        exe: &str,
        action: MediaTransportAction,
    ) -> Result<()> {
        self.platform.send_app_shortcut(hwnd, exe, action)
    }

    pub fn skip_next(&self, id: &str) -> Result<()> {
        self.platform.skip_next(id)
    }

    pub fn skip_previous(&self, id: &str) -> Result<()> {
        self.platform.skip_previous(id)
    }

    pub fn toggle_play_pause(&self, id: &str) -> Result<()> {
        self.platform.toggle_play_pause(id)
    }

    /// `position` in nanoseconds, clamped to the seekable range of the player.
    pub fn seek(&self, id: &str, position: i64) -> Result<()> {
        let (min, max) = seek_range(&self.platform.timeline(id)?);
        self.platform.seek(id, position.clamp(min, max))
    }
}

#[derive(Debug, Clone)]
struct PlayerEntry {
    player: MediaPlayer,
    /// Set while the player is missing, it is released after [`REMOVAL_GRACE`].
    removed_at: Option<Instant>,
}
