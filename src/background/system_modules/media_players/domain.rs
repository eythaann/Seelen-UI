use std::path::PathBuf;

use seelen_core::system_state::{
    MediaPlayer, MediaPlayerControls, MediaPlayerTimeline, MediaTransportAction,
};

use crate::error::Result;

/// OS contract of the media players module (apps exposing media controls).
pub trait MediaPlayersSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening the players list, reported through `on_event`. Each player loaded with
    /// [`Self::load_player`] reports its own changes through it too.
    /// Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(MediaPlayersSysEvent) + Send + Sync + 'static;

    /// Ids of the players currently exposed by the OS.
    fn player_ids(&self) -> Result<Vec<String>>;

    /// Reads the player and starts listening its changes, until [`Self::release_player`].
    fn load_player(&self, id: &str) -> Result<MediaPlayer>;

    fn release_player(&self, id: &str);

    /// The player the OS would control with the media keys.
    fn recommended_player_id(&self) -> Result<String>;

    fn properties(&self, id: &str) -> Result<PlayerProperties>;

    fn is_playing(&self, id: &str) -> Result<bool>;
    fn controls(&self, id: &str) -> Result<MediaPlayerControls>;
    fn send_app_shortcut(&self, hwnd: isize, exe: &str, action: MediaTransportAction)
    -> Result<()>;

    fn timeline(&self, id: &str) -> Result<MediaPlayerTimeline>;

    fn skip_next(&self, id: &str) -> Result<()>;

    fn skip_previous(&self, id: &str) -> Result<()>;

    fn toggle_play_pause(&self, id: &str) -> Result<()>;

    /// `position` in nanoseconds, as the timeline.
    fn seek(&self, id: &str, position: i64) -> Result<()>;
}

/// Raw OS notifications: they only signal what changed, the manager queries the new data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum MediaPlayersSysEvent {
    /// The players list changed, `ids` are the current ones.
    PlayersChanged {
        ids: Vec<String>,
    },
    PropertiesChanged {
        id: String,
    },
    PlaybackChanged {
        id: String,
    },
    TimelineChanged {
        id: String,
    },
}

#[derive(Debug, Clone)]
pub enum MediaPlayersEvent {
    Changed,
}

/// What is being played.
#[derive(Debug, Clone)]
pub struct PlayerProperties {
    pub title: String,
    pub author: String,
    pub thumbnail: Option<PathBuf>,
}

/// Seek range of the timeline, in nanoseconds.
///
/// Not every app populates Min/MaxSeekTime, in that case both default to 0, which would force
/// every seek to position 0, so it falls back to the track's start/end range instead.
pub fn seek_range(timeline: &MediaPlayerTimeline) -> (i64, i64) {
    let (mut min, mut max) = (timeline.min_seek, timeline.max_seek);
    if max <= min {
        (min, max) = (timeline.start, timeline.end);
    }
    // Guard against apps reporting an inverted or degenerate range, which
    // would make `clamp` panic (it requires min <= max).
    (min, max.max(min))
}
