use std::path::PathBuf;

use ts_rs::TS;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export, repr(enum = name)))]
pub enum FolderType {
    Recent,
    Desktop,
    Downloads,
    Documents,
    Music,
    Pictures,
    Videos,
}

static ALL_FOLDERS: [FolderType; 7] = [
    FolderType::Recent,
    FolderType::Desktop,
    FolderType::Downloads,
    FolderType::Documents,
    FolderType::Music,
    FolderType::Pictures,
    FolderType::Videos,
];

impl FolderType {
    pub fn values() -> &'static [FolderType] {
        &ALL_FOLDERS
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct FolderChangedArgs {
    pub of_folder: FolderType,
    pub content: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct User {
    pub name: String,
    pub domain: String,
    pub profile_home_path: PathBuf,
    pub email: Option<String>,
    pub one_drive_path: Option<PathBuf>,
    pub profile_picture_path: Option<PathBuf>,
    pub xbox_gamertag: Option<String>,
}
