use std::path::PathBuf;

use serde::Serialize;
use windows::{
    Media::Control::GlobalSystemMediaTransportControlsSessionManager,
    Storage::Streams::{Buffer, DataReader, InputStreamOptions},
};

use crate::error_handler::Result;

#[derive(Debug, Serialize)]
pub struct MediaSession {
    title: String,
    author: String,
    thumbnail: Option<PathBuf>,
}

pub async fn request_media_sessions() -> Result<Vec<MediaSession>> {
    let mut sessions = Vec::new();
    let session_manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;

    for session in session_manager.GetSessions()? {
        let properties = session.TryGetMediaPropertiesAsync()?.await?;

        let buffer = Buffer::Create(1_000_000)?; // 1MB
        let stream = properties.Thumbnail()?.OpenReadAsync()?.await?;
        stream
            .ReadAsync(&buffer, buffer.Capacity()?, InputStreamOptions::ReadAhead)?
            .await?;

        let reader = DataReader::FromBuffer(&buffer)?;
        let mut bytes = Vec::new();
        while reader.UnconsumedBufferLength()? > 0 {
            bytes.push(reader.ReadByte()?);
        }

        let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)?;
        let image_path = std::env::temp_dir().join(format!("{}.png", uuid::Uuid::new_v4()));
        image.save(&image_path)?;

        sessions.push(MediaSession {
            title: properties.Title()?.to_string_lossy(),
            author: properties.Artist()?.to_string_lossy(),
            thumbnail: Some(image_path),
        });
    }

    Ok(sessions)
}
