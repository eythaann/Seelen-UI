use std::{
    collections::HashMap,
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use seelen_core::{
    handlers::SeelenEvent,
    resource::ResourceText,
    state::{Icon, IconPack},
};
use tauri::Emitter;

use crate::{
    error_handler::Result, seelen::get_app_handle, trace_lock, utils::constants::SEELEN_COMMON,
};

use super::FullState;

#[derive(Debug, Clone, Default)]
pub struct IconPacksManager(HashMap<String, IconPack>);

impl IconPacksManager {
    pub fn list(&self) -> Vec<&IconPack> {
        self.0.values().collect_vec()
    }

    pub fn owned_list(&self) -> Vec<IconPack> {
        self.0.values().cloned().collect_vec()
    }

    pub fn get_system(&self) -> &IconPack {
        self.0.get("system").unwrap()
    }

    pub fn get_system_mut(&mut self) -> &mut IconPack {
        self.0.get_mut("system").unwrap()
    }

    /// key could be path or user model id
    pub fn add_system_app_icon(&mut self, key: &str, target_rel_path: &Path) {
        let system_pack = self.0.get_mut("system").unwrap();
        let key = key.trim_start_matches(r"\\?\").to_string();
        system_pack
            .apps
            .insert(key, Icon::Simple(target_rel_path.to_owned()));
    }

    pub fn add_system_file_icon(&mut self, origin: &Path, target_rel_path: &Path) {
        let system_pack = self.0.get_mut("system").unwrap();
        if let Some(ext) = origin.extension() {
            system_pack.files.insert(
                ext.to_string_lossy().to_lowercase(),
                Icon::Simple(target_rel_path.to_owned()),
            );
        }
    }

    fn resolve_icon_path(&self, icon: &Icon) -> PathBuf {
        match icon {
            Icon::Simple(path) => SEELEN_COMMON
                .icons_path()
                .join(&self.get_system().metadata.filename)
                .join(path),
            Icon::Dynamic { light, dark: _ } => SEELEN_COMMON
                .icons_path()
                .join(&self.get_system().metadata.filename)
                .join(light),
        }
    }

    /// Get icon pack by app user model id, filename or path
    pub fn get_app_icon_by_key(&self, key: &str) -> Option<PathBuf> {
        let filename = PathBuf::from(key)
            .file_name()
            .map(|p| p.to_string_lossy().to_string());

        let icon_pack = self.get_system();
        let maybe_icon = icon_pack.apps.get(key).or_else(|| match filename.as_ref() {
            Some(filename) => icon_pack.apps.get(filename),
            None => None,
        });
        if let Some(icon) = maybe_icon {
            let full_path = self.resolve_icon_path(icon);
            if full_path.exists() {
                return Some(full_path);
            }
        }
        None
    }

    pub fn get_file_icon(&self, path: &Path) -> Option<PathBuf> {
        let extension = path.extension()?.to_string_lossy().to_lowercase();
        let icon_pack = self.get_system();
        if let Some(icon) = icon_pack.files.get(extension.as_str()) {
            let full_path = self.resolve_icon_path(icon);
            if full_path.exists() {
                return Some(full_path);
            }
        }
        None
    }

    pub fn clear_system_icons(&mut self) -> Result<()> {
        let system_pack = self.0.get_mut("system").unwrap();
        system_pack.apps.clear();
        system_pack.files.clear();
        system_pack.specific.clear();
        let meta = std::ffi::OsStr::new("metadata.yml");
        for entry in std::fs::read_dir(SEELEN_COMMON.icons_path().join("system"))?.flatten() {
            if entry.file_type()?.is_dir() {
                std::fs::remove_dir_all(entry.path())?;
            } else if entry.file_name() != meta {
                std::fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }

    pub fn sanitize_system_icon_pack(&mut self) -> Result<()> {
        // add default icon pack if not exists
        if !self.0.contains_key("system") {
            let mut icon_pack = IconPack {
                id: "@system/icon-pack".into(),
                ..Default::default()
            };
            icon_pack.metadata.display_name = ResourceText::En("System".to_string());
            icon_pack.metadata.description =
                ResourceText::En("Icons from Windows and Program Files".to_string());
            icon_pack.metadata.filename = "system".to_string();

            self.0
                .insert(icon_pack.metadata.filename.clone(), icon_pack);
            self.write_system_icon_pack()?;
        }

        let system_pack = self.get_system_mut();
        let missing_path = SEELEN_COMMON.icons_path().join("system/missing-icon.png");
        let start_path = SEELEN_COMMON.icons_path().join("system/start-menu.svg");

        if !missing_path.exists() {
            std::fs::copy(
                SEELEN_COMMON
                    .app_resource_dir()
                    .join("static/icons/missing.png"),
                missing_path,
            )?;
        }

        if !start_path.exists() {
            std::fs::copy(
                SEELEN_COMMON
                    .app_resource_dir()
                    .join("static/icons/start-menu.svg"),
                start_path,
            )?;
        }

        system_pack.missing = Some(Icon::Simple(PathBuf::from("missing-icon.png")));
        system_pack.specific.insert(
            "@seelen/weg::start-menu".to_string(),
            Icon::Simple(PathBuf::from("start-menu.svg")),
        );

        Ok(())
    }

    pub fn write_system_icon_pack(&self) -> Result<()> {
        let folder = SEELEN_COMMON.icons_path().join("system");
        std::fs::create_dir_all(&folder)?;
        let file_path = folder.join("metadata.yml");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)?;
        let system_pack = self.0.get("system").unwrap();
        serde_yaml::to_writer(&mut file, system_pack)?;
        Ok(())
    }
}

impl FullState {
    pub fn emit_icon_packs(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateIconPacksChanged,
            trace_lock!(self.icon_packs()).list(),
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
        let mut icon_packs_manager = trace_lock!(self.icon_packs);
        icon_packs_manager.0.clear();

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || !path.join("metadata.yml").exists() {
                continue;
            }

            let mut icon_pack = match Self::load_icon_pack_from_dir(&path) {
                Ok(icon_pack) => icon_pack,
                Err(err) => {
                    log::error!("Failed to load icon pack ({:?}): {:?}", path, err);
                    continue;
                }
            };

            icon_pack.metadata.filename = entry.file_name().to_string_lossy().to_string();
            icon_packs_manager
                .0
                .insert(icon_pack.metadata.filename.clone(), icon_pack);
        }

        icon_packs_manager.sanitize_system_icon_pack()?;
        Ok(())
    }
}
