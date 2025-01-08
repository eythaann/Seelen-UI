use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use seelen_core::{handlers::SeelenEvent, state::IconPack};
use tauri::Emitter;

use crate::{
    error_handler::Result, seelen::get_app_handle, trace_lock, utils::constants::SEELEN_COMMON,
};

use super::FullState;

impl FullState {
    pub fn emit_icon_packs(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateIconPacksChanged,
            trace_lock!(self.icon_packs()).values().collect_vec(),
        )?;
        Ok(())
    }

    fn load_icon_pack_from_dir(dir_path: &Path) -> Result<IconPack> {
        let file = dir_path.join("metadata.yml");
        if !file.exists() {
            return Err("metadata.yml not found".into());
        }
        Ok(serde_yaml::from_str(&std::fs::read_to_string(&file)?)?)
    }

    pub(super) fn load_icons_packs(&mut self) -> Result<()> {
        let entries = std::fs::read_dir(SEELEN_COMMON.icons_path())?;
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
            self.write_system_icon_pack()?;
        }
        Ok(())
    }

    pub fn write_system_icon_pack(&self) -> Result<()> {
        let folder = SEELEN_COMMON.icons_path().join("system");
        let file_path = folder.join("metadata.yml");
        std::fs::create_dir_all(&folder)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)?;
        let icons = trace_lock!(self.icon_packs);
        let system_pack = icons.get("system").unwrap();
        self.skip_modification(file_path);
        serde_yaml::to_writer(&mut file, system_pack)?;
        Ok(())
    }

    pub fn add_system_icon(&self, key: &str, icon: &Path) {
        let mut icons = trace_lock!(self.icon_packs);
        let system_pack = icons.get_mut("system").unwrap();
        let key: String = key.trim_start_matches(r"\\?\").to_string();
        system_pack.apps.insert(key, icon.to_owned());
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
                    let full_path = SEELEN_COMMON
                        .icons_path()
                        .join(&icon_pack.info.filename)
                        .join(icon);
                    if full_path.exists() {
                        return Some(full_path);
                    }
                }
            }
        }
        None
    }
}
