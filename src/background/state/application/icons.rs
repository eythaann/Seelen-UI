use std::path::{Path, PathBuf};

use seelen_core::state::IconPack;

use crate::{error_handler::Result, trace_lock};

use super::FullState;

impl FullState {
    pub fn icon_packs_folder(&self) -> PathBuf {
        self.data_dir.join("icons")
    }

    fn load_icon_pack_from_dir(dir_path: &Path) -> Result<IconPack> {
        let file = dir_path.join("metadata.yml");
        if !file.exists() {
            return Err("metadata.yml not found".into());
        }
        Ok(serde_yaml::from_str(&std::fs::read_to_string(&file)?)?)
    }

    pub(super) fn load_icons_packs(&mut self) -> Result<()> {
        let entries = std::fs::read_dir(self.icon_packs_folder())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let icon_pack = Self::load_icon_pack_from_dir(&path);
                match icon_pack {
                    Ok(mut icon_pack) => {
                        icon_pack.info.filename = entry.file_name().to_string_lossy().to_string();
                        trace_lock!(self.icon_packs)
                            .insert(icon_pack.info.filename.clone(), icon_pack);
                    }
                    Err(err) => {
                        log::error!("Failed to load icon pack ({:?}): {:?}", path, err)
                    }
                }
            }
        }

        // add default icon pack if not exists
        if trace_lock!(self.icon_packs).get("system").is_none() {
            let mut icon_pack = IconPack::default();
            icon_pack.info.display_name = "System".to_string();
            icon_pack.info.author = "System".to_string();
            icon_pack.info.description = "Icons from Windows and Program Files".to_string();
            icon_pack.info.filename = "system".to_string();
            trace_lock!(self.icon_packs).insert(icon_pack.info.filename.clone(), icon_pack);
        }

        Ok(())
    }

    pub fn push_icon_to_defaults(&self, key: &str, icon: &Path) -> Result<()> {
        let mut icon_packs = trace_lock!(self.icon_packs);
        let default_icon_pack = icon_packs.get_mut("system").unwrap();
        default_icon_pack.apps.insert(
            key.trim_start_matches("\\\\?\\").to_string(),
            icon.to_owned(),
        );

        let metadata_path = self.icon_packs_folder().join("system").join("metadata.yml");
        std::fs::write(&metadata_path, serde_yaml::to_string(default_icon_pack)?)?;
        Ok(())
    }

    /// Get icon pack by app user model id, filename or path
    pub fn get_icon_by_key(&self, key: &str) -> Option<PathBuf> {
        let filename = PathBuf::from(key)
            .file_name()
            .map(|p| p.to_string_lossy().to_string());

        for icon_pack in self.settings.icon_packs.iter().rev() {
            if let Some(icon_pack) = trace_lock!(self.icon_packs).get(icon_pack) {
                let maybe_icon = icon_pack.apps.get(key).or_else(|| match filename.as_ref() {
                    Some(filename) => icon_pack.apps.get(filename),
                    None => None,
                });
                if let Some(icon) = maybe_icon {
                    return Some(
                        self.icon_packs_folder()
                            .join(&icon_pack.info.filename)
                            .join(icon),
                    );
                }
            }
        }
        None
    }
}
