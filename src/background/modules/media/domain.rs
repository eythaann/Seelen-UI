use std::{path::PathBuf, time::Instant};

use seelen_core::system_state::{MediaPlayerOwner, MediaPlayerTimeline};
use serde::Serialize;

use windows::Win32::Media::Audio::{
    AudioObjectType, AudioObjectType_BackCenter, AudioObjectType_BackLeft,
    AudioObjectType_BackRight, AudioObjectType_BottomBackLeft, AudioObjectType_BottomBackRight,
    AudioObjectType_BottomFrontLeft, AudioObjectType_BottomFrontRight, AudioObjectType_FrontCenter,
    AudioObjectType_FrontLeft, AudioObjectType_FrontRight, AudioObjectType_LowFrequency,
    AudioObjectType_SideLeft, AudioObjectType_SideRight, AudioObjectType_TopBackLeft,
    AudioObjectType_TopBackRight, AudioObjectType_TopFrontLeft, AudioObjectType_TopFrontRight,
    Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback},
    IAudioSessionControl2, IAudioSessionEvents, IAudioSessionManager2, IAudioSessionNotification,
};

#[allow(dead_code)]
pub struct ChannelMask {}

#[allow(non_upper_case_globals)]
#[allow(dead_code)]
impl ChannelMask {
    pub const Mono: AudioObjectType = AudioObjectType_FrontCenter;
    pub const Stereo: AudioObjectType =
        AudioObjectType(AudioObjectType_FrontLeft.0 | AudioObjectType_FrontRight.0);

    pub const Spatial_2_1: AudioObjectType =
        AudioObjectType(Self::Stereo.0 | AudioObjectType_LowFrequency.0);
    pub const Quad: AudioObjectType =
        AudioObjectType(Self::Stereo.0 | AudioObjectType_BackLeft.0 | AudioObjectType_BackRight.0);
    pub const Spatial_4_1: AudioObjectType =
        AudioObjectType(Self::Quad.0 | AudioObjectType_LowFrequency.0);
    pub const Spatial_5_1: AudioObjectType = AudioObjectType(
        Self::Stereo.0
            | AudioObjectType_FrontCenter.0
            | AudioObjectType_LowFrequency.0
            | AudioObjectType_SideLeft.0
            | AudioObjectType_SideRight.0,
    );

    pub const Spatial_7_1: AudioObjectType = AudioObjectType(
        Self::Spatial_5_1.0 | AudioObjectType_BackLeft.0 | AudioObjectType_BackRight.0,
    );

    pub const MaxStaticObjectCount_7_1_4: u32 = 12;
    pub const Spatial_7_1_4: AudioObjectType = AudioObjectType(
        Self::Spatial_7_1.0
            | AudioObjectType_TopFrontLeft.0
            | AudioObjectType_TopFrontRight.0
            | AudioObjectType_TopBackLeft.0
            | AudioObjectType_TopBackRight.0,
    );

    pub const MaxStaticObjectCount_7_1_4_4: u32 = 16;
    pub const Spatial_7_1_4_4: AudioObjectType = AudioObjectType(
        Self::Spatial_7_1_4.0
            | AudioObjectType_BottomFrontLeft.0
            | AudioObjectType_BottomFrontRight.0
            | AudioObjectType_BottomBackLeft.0
            | AudioObjectType_BottomBackRight.0,
    );

    pub const MaxStaticObjectCount_8_1_4_4: u32 = 17;
    pub const Spatial_8_1_4_4: AudioObjectType =
        AudioObjectType(Self::Spatial_7_1_4_4.0 | AudioObjectType_BackCenter.0);
}

#[derive(Debug, Clone, Serialize)]
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
    #[serde(skip)]
    pub removed_at: Option<Instant>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDeviceSession {
    #[serde(skip)]
    pub controls: IAudioSessionControl2,
    #[serde(skip)]
    pub events_callback: IAudioSessionEvents,
    // ---
    pub id: String,
    pub instance_id: String,
    pub process_id: u32,
    pub name: String,
    pub icon_path: Option<PathBuf>,
    pub is_system: bool,
    pub volume: f32,
    pub muted: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaDeviceType {
    Input,
    Output,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDevice {
    #[serde(skip)]
    pub volume_endpoint: IAudioEndpointVolume,
    #[serde(skip)]
    pub volume_callback: IAudioEndpointVolumeCallback,
    #[serde(skip)]
    pub session_manager: IAudioSessionManager2,
    #[serde(skip)]
    pub session_created_callback: IAudioSessionNotification,
    // ---
    pub id: String,
    pub name: String,
    pub r#type: MediaDeviceType,
    pub is_default_multimedia: bool,
    pub is_default_communications: bool,
    pub sessions: Vec<MediaDeviceSession>,
    pub volume: f32,
    pub muted: bool,
}
