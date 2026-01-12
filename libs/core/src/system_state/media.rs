use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MediaPlayerOwner {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MediaPlayerTimeline {
    /// The starting timestamp in nanoseconds (aparently it's always 0)
    pub start: i64,
    /// The total duration of the media item in nanoseconds
    pub end: i64,
    /// Current playback position in nanoseconds
    pub position: i64,
    /// The earliest timestamp at which the current media item can currently seek to. (in nanoseconds)
    pub min_seek: i64,
    /// The furthest timestamp at which the content can currently seek to. (in nanoseconds)
    pub max_seek: i64,
    pub last_updated_time: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MediaPlayer {
    pub umid: String,
    pub title: String,
    pub author: String,
    pub thumbnail: Option<PathBuf>,
    pub owner: MediaPlayerOwner,
    pub timeline: MediaPlayerTimeline,
    pub playing: bool,
    pub default: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MediaDeviceSession {
    pub id: String,
    pub instance_id: String,
    pub process_id: u32,
    pub name: String,
    pub icon_path: Option<PathBuf>,
    pub is_system: bool,
    pub volume: f32,
    pub muted: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(repr(enum = name))]
pub enum MediaDeviceType {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MediaDevice {
    pub id: String,
    pub name: String,
    pub r#type: MediaDeviceType,
    pub is_default_multimedia: bool,
    pub is_default_communications: bool,
    pub sessions: Vec<MediaDeviceSession>,
    pub volume: f32,
    pub muted: bool,
}
