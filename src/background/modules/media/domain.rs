use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MediaSession {
    pub id: String,
    pub title: String,
    pub author: String,
    pub thumbnail: Option<PathBuf>,
    pub playing: bool,
    pub default: bool,
}
