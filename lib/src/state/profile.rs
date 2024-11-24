use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::Placeholder;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSettings {
    themes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    name: String,
    toolbar_layout: Placeholder,
    settings: ProfileSettings,
}

impl Profile {
    pub fn load(profile_dir: &Path) -> Result<Profile> {
        if !profile_dir.is_dir() {
            return Err("Invalid profile path".into());
        }
        let profile = Profile {
            name: profile_dir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            toolbar_layout: serde_yaml::from_reader(std::fs::File::open(
                profile_dir.join("toolbar.yml"),
            )?)?,
            settings: serde_json::from_reader(std::fs::File::open(
                profile_dir.join("settings.json"),
            )?)?,
        };
        Ok(profile)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let folder = path.join(&self.name);
        std::fs::create_dir_all(&folder)?;
        std::fs::write(
            folder.join("toolbar.yml"),
            serde_yaml::to_string(&self.toolbar_layout)?,
        )?;
        std::fs::write(
            folder.join("settings.json"),
            serde_json::to_string(&self.settings)?,
        )?;
        Ok(())
    }
}
