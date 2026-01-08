use std::time::Instant;

use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession, MediaPropertiesChangedEventArgs,
        PlaybackInfoChangedEventArgs, TimelinePropertiesChangedEventArgs,
    },
};

use crate::error::{Result, ResultLogExt};

/// Wrapper for GlobalSystemMediaTransportControlsSession that automatically
/// registers and unregisters event handlers on creation and drop
pub struct MediaPlayerSession {
    pub session: GlobalSystemMediaTransportControlsSession,
    properties_token: i64,
    playback_token: i64,
    timeline_token: i64,
}

impl MediaPlayerSession {
    pub fn create(
        session: GlobalSystemMediaTransportControlsSession,
        properties_handler: &TypedEventHandler<
            GlobalSystemMediaTransportControlsSession,
            MediaPropertiesChangedEventArgs,
        >,
        playback_handler: &TypedEventHandler<
            GlobalSystemMediaTransportControlsSession,
            PlaybackInfoChangedEventArgs,
        >,
        timeline_handler: &TypedEventHandler<
            GlobalSystemMediaTransportControlsSession,
            TimelinePropertiesChangedEventArgs,
        >,
    ) -> Result<Self> {
        // WinRT event tokens are i64, not EventRegistrationToken
        let properties_token = session.MediaPropertiesChanged(properties_handler)?;
        let playback_token = session.PlaybackInfoChanged(playback_handler)?;
        let timeline_token = session.TimelinePropertiesChanged(timeline_handler)?;

        Ok(Self {
            session,
            properties_token,
            playback_token,
            timeline_token,
        })
    }
}

impl Drop for MediaPlayerSession {
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

#[derive(Debug, Clone)]
pub struct MediaPlayer {
    pub base: seelen_core::system_state::MediaPlayer,
    pub removed_at: Option<Instant>,
}

impl serde::Serialize for MediaPlayer {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.base.serialize(serializer)
    }
}
