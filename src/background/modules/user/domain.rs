use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error_handler::AppError;

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

const RECENT_FOLDER_RELATIVE_PATH: &str = "..\\..\\Roaming\\Microsoft\\Windows\\Recent";
const DOCUMENTS_FOLDER_RELATIVE_PATH: &str = "..\\..\\..\\Documents";
const DOWNLOADS_RELATIVE_PATH: &str = "..\\..\\..\\Downloads";
const PICTURES_RELATIVE_PATH: &str = "..\\..\\..\\Pictures";
const VIDEOS_RELATIVE_PATH: &str = "..\\..\\..\\Videos";
const MUSIC_RELATIVE_PATH: &str = "..\\..\\..\\Music";

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum FolderType {
    Recent,
    Downloads,
    Documents,
    Pictures,
    Videos,
    Music,
}

impl FolderType {
    pub fn to_path(&self) -> PathBuf {
        match self {
            FolderType::Recent => std::env::temp_dir().join(RECENT_FOLDER_RELATIVE_PATH),
            FolderType::Downloads => std::env::temp_dir().join(DOWNLOADS_RELATIVE_PATH),
            FolderType::Documents => std::env::temp_dir().join(DOCUMENTS_FOLDER_RELATIVE_PATH),
            FolderType::Pictures => std::env::temp_dir().join(PICTURES_RELATIVE_PATH),
            FolderType::Videos => std::env::temp_dir().join(VIDEOS_RELATIVE_PATH),
            FolderType::Music => std::env::temp_dir().join(MUSIC_RELATIVE_PATH),
        }
    }

    pub fn values() -> [FolderType; 6] {
        [
            FolderType::Recent,
            FolderType::Downloads,
            FolderType::Documents,
            FolderType::Pictures,
            FolderType::Videos,
            FolderType::Music,
        ]
    }

    pub fn from_path(path: &PathBuf) -> Result<FolderType, AppError> {
        for folder_type in FolderType::values() {
            if path.starts_with(folder_type.to_path()) {
                return Ok(folder_type);
            }
        }

        Err("No folder type found!".into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct File {
    pub path: PathBuf,
    pub last_access_time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderChangedArgs {
    pub of_folder: FolderType,
    pub content: Option<Vec<File>>,
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
