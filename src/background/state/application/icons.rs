use std::{
    collections::HashMap,
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use seelen_core::{
    handlers::SeelenEvent,
    resource::ResourceText,
    state::{AppIconPackEntry, CustomIconPackEntry, FileIconPackEntry, Icon, IconPack},
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

    pub fn add_system_app_icon(&mut self, umid: Option<&str>, path: Option<&Path>, icon: Icon) {
        if umid.is_none() && path.is_none() {
            return;
        }
        let system_pack = self.get_system_mut();
        system_pack.app_entries.push(AppIconPackEntry {
            umid: umid.map(|s| s.to_string()),
            path: path.map(|p| p.to_path_buf()),
            redirect: None,
            icon: Some(icon),
        });
    }

    pub fn add_system_icon_redirect(
        &mut self,
        umid: Option<String>,
        origin: &Path,
        redirect: &Path,
    ) {
        let system_pack = self.get_system_mut();
        system_pack.app_entries.push(AppIconPackEntry {
            umid,
            path: Some(origin.to_path_buf()),
            redirect: Some(redirect.to_path_buf()),
            icon: None,
        });
    }

    pub fn add_system_file_icon(&mut self, origin_extension: &str, icon: Icon) {
        let system_pack = self.get_system_mut();
        system_pack.file_entries.push(FileIconPackEntry {
            extension: origin_extension.to_string(),
            icon,
        });
    }

    fn icon_exists(&self, icon: &Icon) -> bool {
        let root = SEELEN_COMMON.user_icons_path().join("system");
        icon.base.as_ref().is_some_and(|p| root.join(p).exists())
            || (icon.light.as_ref().is_some_and(|p| root.join(p).exists())
                && icon.dark.as_ref().is_some_and(|p| root.join(p).exists()))
    }

    /// Get icon pack by app user model id, filename or path
    pub fn has_app_icon(&self, umid: Option<&str>, path: Option<&Path>) -> bool {
        let icon_pack = self.get_system();
        let lower_path = path.map(|p| p.to_string_lossy().to_lowercase());

        for entry in &icon_pack.app_entries {
            let mut found = None;
            if let (Some(entry_umid), Some(umid)) = (&entry.umid, umid) {
                if entry_umid == umid {
                    found = Some(entry);
                }
            }

            if found.is_none()
                && entry
                    .path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_lowercase())
                    == lower_path
            {
                found = Some(entry);
            }

            if let Some(entry) = found {
                if entry.redirect.is_some() {
                    return true;
                }

                if let Some(icon) = &entry.icon {
                    if self.icon_exists(icon) {
                        return true;
                    }
                }
            };
        }

        false
    }

    pub fn get_file_icon(&self, path: &Path) -> Option<&Icon> {
        let extension = path.extension()?.to_string_lossy().to_lowercase();
        let icon_pack = self.get_system();
        if let Some(entry) = icon_pack
            .file_entries
            .iter()
            .find(|e| e.extension.to_lowercase() == extension)
        {
            if self.icon_exists(&entry.icon) {
                return Some(&entry.icon);
            }
        }
        None
    }

    pub fn clear_system_icons(&mut self) -> Result<()> {
        let system_pack = self.get_system_mut();
        system_pack.app_entries.clear();
        system_pack.file_entries.clear();
        system_pack.custom_entries.clear();
        let meta = std::ffi::OsStr::new("metadata.yml");
        for entry in std::fs::read_dir(SEELEN_COMMON.user_icons_path().join("system"))?.flatten() {
            if entry.file_type()?.is_dir() {
                std::fs::remove_dir_all(entry.path())?;
            } else if entry.file_name() != meta {
                std::fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }

    pub fn sanitize_system_icon_pack(&mut self, initial: bool) -> Result<()> {
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
        let missing_path = SEELEN_COMMON
            .user_icons_path()
            .join("system/missing-icon.png");
        let start_path = SEELEN_COMMON
            .user_icons_path()
            .join("system/start-menu-icon.svg");
        let folder_path = SEELEN_COMMON
            .user_icons_path()
            .join("system/folder-icon.svg");
        let url_path = SEELEN_COMMON.user_icons_path().join("system/url.png");

        if !missing_path.exists() || initial {
            std::fs::copy(
                SEELEN_COMMON
                    .app_resource_dir()
                    .join("static/icons/missing.png"),
                missing_path,
            )?;
        }

        if !start_path.exists() || initial {
            std::fs::copy(
                SEELEN_COMMON
                    .app_resource_dir()
                    .join("static/icons/start-menu.svg"),
                start_path,
            )?;
        }

        if !folder_path.exists() || initial {
            std::fs::copy(
                SEELEN_COMMON
                    .app_resource_dir()
                    .join("static/icons/folder.svg"),
                folder_path,
            )?;
        }

        if !url_path.exists() || initial {
            std::fs::copy(
                SEELEN_COMMON
                    .app_resource_dir()
                    .join("static/icons/url.png"),
                url_path,
            )?;
        }

        system_pack.missing = Some(Icon {
            base: Some(PathBuf::from("missing-icon.png")),
            ..Default::default()
        });

        if !system_pack
            .file_entries
            .iter()
            .any(|e| e.extension == "url")
        {
            system_pack.file_entries.push(FileIconPackEntry {
                extension: "url".to_string(),
                icon: Icon {
                    base: Some(PathBuf::from("url.png")),
                    ..Default::default()
                },
            });
        }

        let mut add_if_none = |key: &str, value: &str| {
            if !system_pack.custom_entries.iter().any(|e| e.key == key) {
                system_pack.custom_entries.push(CustomIconPackEntry {
                    key: key.to_owned(),
                    icon: Icon {
                        base: Some(PathBuf::from(value)),
                        mask: Some(PathBuf::from(value)),
                        ..Default::default()
                    },
                });
            };
        };

        add_if_none("@seelen/weg::start-menu", "start-menu-icon.svg");
        add_if_none("@seelen/weg::folder", "folder-icon.svg");
        Ok(())
    }

    pub fn write_system_icon_pack(&self) -> Result<()> {
        let folder = SEELEN_COMMON.user_icons_path().join("system");
        std::fs::create_dir_all(&folder)?;
        let file_path = folder.join("metadata.yml");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)?;
        let system_pack = self.get_system();
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

    pub(super) fn load_icons_packs(&mut self, initial: bool) -> Result<()> {
        let entries = std::fs::read_dir(SEELEN_COMMON.user_icons_path())?;
        let mut icon_packs_manager = trace_lock!(self.icon_packs);
        icon_packs_manager.0.clear();

        for entry in entries.flatten() {
            let path = entry.path();
            let mut icon_pack = match IconPack::load(&path) {
                Ok(icon_pack) => icon_pack,
                Err(err) => {
                    log::error!("Failed to load icon pack ({path:?}): {err:?}");
                    continue;
                }
            };
            icon_pack.metadata.bundled = entry.file_name() == "system";
            icon_pack.metadata.filename = entry.file_name().to_string_lossy().to_string();
            icon_packs_manager
                .0
                .insert(icon_pack.metadata.filename.clone(), icon_pack);
        }

        icon_packs_manager.sanitize_system_icon_pack(initial)?;
        Ok(())
    }
}
