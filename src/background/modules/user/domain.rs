use std::path::PathBuf;

use serde::Serialize;

use crate::{seelen_weg::icon_extractor, utils::constants::Icons};

#[allow(dead_code)]
pub enum PictureQuality {
    Quality1080,
    Quality448,
    Quality424,
    Quality240,
    Quality208,
    Quality192,
    Quality96,
    Quality64,
    Quality48,
    Quality40,
    Quality32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecentFile {
    pub path: PathBuf,
    pub last_access_time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExposedRecentFile {
    pub name: String,
    pub path: PathBuf,
    pub icon_location: PathBuf,
    pub last_access_time: u64,
}

impl From<RecentFile> for ExposedRecentFile {
    fn from(value: RecentFile) -> ExposedRecentFile {
        ExposedRecentFile {
            name: value
                .path
                .clone()
                .as_path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            path: value.path.clone(),
            icon_location: std::fs::canonicalize(
                icon_extractor::extract_and_save_icon_from_file(&value.path)
                    .unwrap_or(Icons::missing_app()),
            )
            .unwrap()
            .to_str()
            .unwrap()[4..]
                .into(),
            last_access_time: value.last_access_time,
        }
    }
}

impl PictureQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            PictureQuality::Quality1080 => "Image1080",
            PictureQuality::Quality192 => "Image192",
            PictureQuality::Quality208 => "Image208",
            PictureQuality::Quality240 => "Image240",
            PictureQuality::Quality32 => "Image32",
            PictureQuality::Quality40 => "Image40",
            PictureQuality::Quality424 => "Image424",
            PictureQuality::Quality448 => "Image448",
            PictureQuality::Quality48 => "Image48",
            PictureQuality::Quality64 => "Image64",
            PictureQuality::Quality96 => "Image96",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub name: String,
    pub domain: String,
    pub profile_home_path: PathBuf,
    pub email: Option<String>,
    pub one_drive_path: Option<PathBuf>,
    pub profile_picture_path: Option<PathBuf>,
}
