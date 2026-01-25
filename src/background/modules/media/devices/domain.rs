use std::path::PathBuf;

use serde::Serialize;
use windows::Win32::Media::Audio::{
    Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback},
    IAudioSessionControl2, IAudioSessionEvents, IAudioSessionManager2, IAudioSessionNotification,
};

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

impl MediaDevice {
    pub fn session(&self, session_id: &str) -> Option<&MediaDeviceSession> {
        self.sessions.iter().find(|s| s.id == session_id)
    }

    pub fn session_mut(&mut self, session_id: &str) -> Option<&mut MediaDeviceSession> {
        self.sessions.iter_mut().find(|s| s.id == session_id)
    }

    pub fn remove_session(&mut self, session_id: &str) {
        for session in std::mem::take(&mut self.sessions) {
            if session.id == session_id {
                session.release();
                continue;
            }
            self.sessions.push(session);
        }
    }

    pub fn release(&self) {
        unsafe {
            use crate::error::ResultLogExt;
            self.volume_endpoint
                .UnregisterControlChangeNotify(&self.volume_callback)
                .log_error();
            self.session_manager
                .UnregisterSessionNotification(&self.session_created_callback)
                .log_error();
        };
    }
}

impl MediaDeviceSession {
    pub fn release(self) {
        unsafe {
            use crate::error::ResultLogExt;
            self.controls
                .UnregisterAudioSessionNotification(&self.events_callback)
                .log_error();
        }
    }
}
