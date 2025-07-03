use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use itertools::Itertools;
use seelen_core::{
    handlers::SeelenEvent,
    resource::{ResourceText, SluResource},
    state::{
        CustomIconPackEntry, Icon, IconPack, IconPackEntry, SharedIconPackEntry,
        UniqueIconPackEntry,
    },
};
use tauri::Emitter;

use crate::{
    error_handler::Result,
    seelen::get_app_handle,
    trace_lock,
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
};

use super::FullState;

static SYSTEM_ICONS: LazyLock<PathBuf> =
    LazyLock::new(|| SEELEN_COMMON.user_icons_path().join("system"));

#[derive(Debug, Clone, Default)]
pub struct IconPacksManager(HashMap<PathBuf, IconPack>);

impl IconPacksManager {
    pub fn list(&self) -> Vec<&IconPack> {
        self.0.values().collect_vec()
    }

    pub fn owned_list(&self) -> Vec<IconPack> {
        self.0.values().cloned().collect_vec()
    }

    pub fn get_system(&self) -> &IconPack {
        self.0.get(SYSTEM_ICONS.as_path()).unwrap()
    }

    pub fn get_system_mut(&mut self) -> &mut IconPack {
        self.0.get_mut(SYSTEM_ICONS.as_path()).unwrap()
    }

    pub fn add_system_app_icon(&mut self, umid: Option<&str>, path: Option<&Path>, icon: Icon) {
        if umid.is_none() && path.is_none() {
            return;
        }
        let system_pack = self.get_system_mut();
        system_pack.add_entry(IconPackEntry::Unique(UniqueIconPackEntry {
            umid: umid.map(|s| s.to_string()),
            path: path.map(|p| p.to_path_buf()),
            redirect: None,
            icon: Some(icon),
        }));
    }

    pub fn add_system_icon_redirect(
        &mut self,
        umid: Option<String>,
        origin: &Path,
        redirect: &Path,
    ) {
        let system_pack = self.get_system_mut();
        system_pack.add_entry(IconPackEntry::Unique(UniqueIconPackEntry {
            umid,
            path: Some(origin.to_path_buf()),
            redirect: Some(redirect.to_path_buf()),
            icon: None,
        }));
    }

    pub fn add_system_file_icon(&mut self, origin_extension: &str, icon: Icon) {
        let system_pack = self.get_system_mut();
        system_pack.add_entry(IconPackEntry::Shared(SharedIconPackEntry {
            extension: origin_extension.to_string(),
            icon,
        }));
    }

    fn icon_exists(&self, icon: &Icon) -> bool {
        icon.base
            .as_ref()
            .is_some_and(|p| SYSTEM_ICONS.join(p).exists())
            || (icon
                .light
                .as_ref()
                .is_some_and(|p| SYSTEM_ICONS.join(p).exists())
                && icon
                    .dark
                    .as_ref()
                    .is_some_and(|p| SYSTEM_ICONS.join(p).exists()))
    }

    /// Get icon pack by app user model id, filename or path
    pub fn has_app_icon(&self, umid: Option<&str>, path: Option<&Path>) -> bool {
        let icon_pack = self.get_system();
        let lower_path = path.map(|p| p.to_string_lossy().to_lowercase());

        for entry in &icon_pack.entries {
            let IconPackEntry::Unique(entry) = entry else {
                continue;
            };

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
        for entry in &icon_pack.entries {
            match entry {
                IconPackEntry::Shared(entry) if entry.extension.to_lowercase() == extension => {
                    if self.icon_exists(&entry.icon) {
                        return Some(&entry.icon);
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn clear_system_icons(&mut self) -> Result<()> {
        let system_pack = self.get_system_mut();
        system_pack.entries.clear();
        let meta = std::ffi::OsStr::new("metadata.yml");
        for entry in std::fs::read_dir(SYSTEM_ICONS.as_path())?.flatten() {
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
        if !self.0.contains_key(SYSTEM_ICONS.as_path()) {
            let mut icon_pack = IconPack {
                id: "@system/icon-pack".into(),
                ..Default::default()
            };
            icon_pack.metadata.display_name = ResourceText::En("System".to_string());
            icon_pack.metadata.description =
                ResourceText::En("Icons from Windows and Program Files".to_string());
            icon_pack.metadata.path = SYSTEM_ICONS.to_path_buf();

            self.0.insert(icon_pack.metadata.path.clone(), icon_pack);
            self.write_system_icon_pack()?;
        }

        let system_pack = self.get_system_mut();
        let missing_path = SYSTEM_ICONS.join("missing-icon.png");
        let start_path = SYSTEM_ICONS.join("start-menu-icon.svg");
        let folder_path = SYSTEM_ICONS.join("folder-icon.svg");
        let url_path = SYSTEM_ICONS.join("url.png");

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
            base: Some("missing-icon.png".to_owned()),
            ..Default::default()
        });

        system_pack.add_entry(IconPackEntry::Shared(SharedIconPackEntry {
            extension: "url".to_string(),
            icon: Icon {
                base: Some("url.png".to_owned()),
                ..Default::default()
            },
        }));

        let mut add_custom = |key: &str, value: &str| {
            system_pack.add_entry(IconPackEntry::Custom(CustomIconPackEntry {
                key: key.to_owned(),
                icon: Icon {
                    base: Some(value.to_owned()),
                    mask: Some(value.to_owned()),
                    ..Default::default()
                },
            }));
        };

        add_custom("@seelen/weg::start-menu", "start-menu-icon.svg");
        add_custom("@seelen/weg::folder", "folder-icon.svg");
        Ok(())
    }

    pub fn write_system_icon_pack(&self) -> Result<()> {
        self.get_system().save()?;
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

    // download remote icon url and save it on the parent path + random hash.
    fn load_remote_icon(icon: &Icon, folder_to_store: &Path) -> Result<Icon> {
        let mut resolved = icon.clone();

        let download_filename = |url: &str| -> Result<String> {
            Ok(download_remote_icon_and_validate_it(url, folder_to_store)?
                .file_name()
                .ok_or("Could not get file name")?
                .to_string_lossy()
                .to_string())
        };

        if let Some(url) = &icon.base {
            resolved.base = download_filename(url).ok();
        }

        if let Some(url) = &icon.light {
            resolved.light = download_filename(url).ok();
        }

        if let Some(url) = &icon.dark {
            resolved.dark = download_filename(url).ok()
        }

        if let Some(url) = &icon.mask {
            resolved.mask = download_filename(url).ok();
        }

        Ok(resolved)
    }

    fn load_remote_icons(pack: &mut IconPack) -> Result<()> {
        if pack.remote_entries.is_empty() || pack.downloaded {
            return Ok(());
        }

        let mut entries = Vec::new();

        for entry in &pack.remote_entries {
            let mut new_entry = entry.clone();

            match &mut new_entry {
                IconPackEntry::Unique(entry) => {
                    if let Some(icon) = &mut entry.icon {
                        *icon = Self::load_remote_icon(icon, &pack.metadata.path)?;
                    }
                }
                IconPackEntry::Shared(entry) => {
                    entry.icon = Self::load_remote_icon(&entry.icon, &pack.metadata.path)?;
                }
                IconPackEntry::Custom(entry) => {
                    entry.icon = Self::load_remote_icon(&entry.icon, &pack.metadata.path)?;
                }
            }

            entries.push(new_entry);
        }

        pack.entries = entries;
        pack.downloaded = true;
        pack.save()?;
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
            if let Err(err) = Self::load_remote_icons(&mut icon_pack) {
                log::error!("Failed to load remote icons for icon pack ({path:?}): {err:?}");
                continue;
            }

            icon_packs_manager
                .0
                .insert(icon_pack.metadata.path.clone(), icon_pack);
        }

        icon_packs_manager.sanitize_system_icon_pack(initial)?;
        Ok(())
    }
}

/// returns a path to the downloaded icon
fn download_remote_icon_and_validate_it(url: &str, folder_to_store: &Path) -> Result<PathBuf> {
    if !folder_to_store.is_dir() {
        return Err("Folder to store is not a directory".into());
    }

    let bytes = tauri::async_runtime::block_on(async move {
        let res = reqwest::get(url).await?;
        res.bytes().await
    })?;

    let format = image::guess_format(&bytes)?;
    let icon = image::load_from_memory_with_format(&bytes, format)?;
    let extension = format
        .extensions_str()
        .first()
        .ok_or("Could not get extension")?;

    let icon_path = folder_to_store.join(format!("{}.{}", date_based_hex_id(), extension));
    icon.save(&icon_path)?;
    Ok(icon_path)
}
