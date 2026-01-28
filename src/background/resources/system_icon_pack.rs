use std::{path::Path, sync::LazyLock};

use seelen_core::chrono::{DateTime, Utc};
use seelen_core::{
    resource::{ResourceText, SluResource},
    state::{
        CustomIconPackEntry, Icon, IconPack, IconPackEntry, SharedIconPackEntry,
        UniqueIconPackEntry,
    },
};

use crate::{
    error::{Result, ResultLogExt},
    resources::{ResourceManager, RESOURCES},
    utils::constants::SEELEN_COMMON,
};

static SAVE_SYSTEM_ICON_PACK: LazyLock<slu_utils::Throttle<()>> = LazyLock::new(|| {
    slu_utils::throttle(
        |()| {
            RESOURCES.with_system_pack(|pack| pack.save()).log_error();
        },
        std::time::Duration::from_secs(1),
    )
});

impl ResourceManager {
    fn with_system_pack<F, T>(&self, cb: F) -> T
    where
        F: FnOnce(&mut IconPack) -> T,
    {
        let mut guard = self.system_icon_pack.lock();
        cb(guard
            .as_mut()
            .expect("System icon pack should always exist."))
    }

    fn request_save_system_icon_pack(&self) {
        SAVE_SYSTEM_ICON_PACK.call(());
    }

    /// Ensures default icons exist in the system icon pack directory
    fn sanitize_default_icons(sys_icons_path: &Path) -> Result<()> {
        std::fs::create_dir_all(sys_icons_path)?;

        let ensure_icon = |filename: &str| {
            let icon_path = sys_icons_path.join(filename);
            if !icon_path.exists() {
                std::fs::copy(
                    SEELEN_COMMON
                        .app_resource_dir()
                        .join("static/icons")
                        .join(filename),
                    icon_path,
                )?;
            }
            Result::Ok(())
        };

        ensure_icon("missing.png")?;
        ensure_icon("music_thumbnail.jpg")?;
        ensure_icon("url.png")?;
        ensure_icon("start-menu.svg")?;
        ensure_icon("folder.svg")?;
        ensure_icon("desktop.svg")?;
        Ok(())
    }

    /// Ensures default icon entries exist in the icon pack
    fn sanitize_default_entries(system_pack: &mut IconPack) {
        // Ensure missing icon is set
        system_pack.missing = Some(Icon {
            base: Some("missing.png".to_owned()),
            ..Default::default()
        });

        // add_entry will override if exists, or create if not
        system_pack.add_entry(IconPackEntry::Shared(SharedIconPackEntry {
            extension: "url".to_string(),
            icon: Icon {
                base: Some("url.png".to_owned()),
                ..Default::default()
            },
        }));

        system_pack.add_entry(IconPackEntry::Custom(CustomIconPackEntry {
            key: "@seelen/weg::start-menu".to_owned(),
            icon: Icon {
                base: Some("start-menu.svg".to_owned()),
                mask: Some("start-menu.svg".to_owned()),
                ..Default::default()
            },
        }));

        system_pack.add_entry(IconPackEntry::Custom(CustomIconPackEntry {
            key: "@seelen/weg::folder".to_owned(),
            icon: Icon {
                base: Some("folder.svg".to_owned()),
                mask: Some("folder.svg".to_owned()),
                ..Default::default()
            },
        }));

        system_pack.add_entry(IconPackEntry::Custom(CustomIconPackEntry {
            key: "@seelen/weg::show-desktop".to_owned(),
            icon: Icon {
                base: Some("desktop.svg".to_owned()),
                ..Default::default()
            },
        }));

        system_pack.add_entry(IconPackEntry::Custom(CustomIconPackEntry {
            key: "defaultPlayerThumbnail".to_owned(),
            icon: Icon {
                base: Some("music_thumbnail.jpg".to_owned()),
                ..Default::default()
            },
        }));
    }

    pub fn ensure_system_icon_pack(&self) -> Result<()> {
        let sys_icons_path = SEELEN_COMMON.system_icon_pack_path();

        let mut guard = self.system_icon_pack.lock();
        // Create new pack if it doesn't exist
        if guard.is_none() {
            let mut system_pack = IconPack {
                id: "@system/icon-pack".into(),
                ..Default::default()
            };
            system_pack.metadata.display_name = ResourceText::En("System".to_string());
            system_pack.metadata.description =
                ResourceText::En("Icons from Windows and Program Files".to_string());
            system_pack.metadata.internal.path = sys_icons_path.to_path_buf();

            *guard = Some(system_pack);
        }

        // Always sanitize default icon entries and files
        let system_pack = guard.as_mut().expect("System icon pack should exist");
        Self::sanitize_default_entries(system_pack);
        Self::sanitize_default_icons(sys_icons_path)?;

        self.request_save_system_icon_pack();
        Ok(())
    }

    pub fn add_system_app_icon(&self, umid: Option<&str>, path: Option<&Path>, icon: Icon) {
        if umid.is_none() && path.is_none() {
            return;
        }

        let source_mtime = path.and_then(last_edit_at);
        self.with_system_pack(|system_pack| {
            system_pack.add_entry(IconPackEntry::Unique(UniqueIconPackEntry {
                umid: umid.map(|s| s.to_string()),
                path: path.map(|p| p.to_path_buf()),
                redirect: None,
                icon: Some(icon),
                source_mtime,
            }));
        });
        self.request_save_system_icon_pack();
        self.emit_icon_packs().log_error();
    }

    pub fn add_system_icon_redirect(&self, umid: Option<String>, origin: &Path, redirect: &Path) {
        let source_mtime = last_edit_at(origin);
        self.with_system_pack(|system_pack| {
            system_pack.add_entry(IconPackEntry::Unique(UniqueIconPackEntry {
                umid,
                path: Some(origin.to_path_buf()),
                redirect: Some(redirect.to_path_buf()),
                icon: None,
                source_mtime,
            }));
        });
        self.request_save_system_icon_pack();
        self.emit_icon_packs().log_error();
    }

    pub fn add_system_file_icon(&self, origin_extension: &str, icon: Icon) {
        self.with_system_pack(|system_pack| {
            system_pack.add_entry(IconPackEntry::Shared(SharedIconPackEntry {
                extension: origin_extension.to_string(),
                icon,
            }));
        });
        self.request_save_system_icon_pack();
        self.emit_icon_packs().log_error();
    }

    fn icon_exists(icon: &Icon) -> bool {
        let root_path = SEELEN_COMMON.system_icon_pack_path();
        icon.base
            .as_ref()
            .is_some_and(|sub| root_path.join(sub).exists())
            || (icon
                .light
                .as_ref()
                .is_some_and(|sub| root_path.join(sub).exists())
                && icon
                    .dark
                    .as_ref()
                    .is_some_and(|sub| root_path.join(sub).exists()))
    }

    /// Internal recursive function that checks for app icon without acquiring locks
    fn _has_app_icon(system_pack: &IconPack, umid: Option<&str>, path: Option<&Path>) -> bool {
        let current_mtime = path.and_then(last_edit_at);
        let lower_path = path.map(|p| p.to_string_lossy().to_lowercase());

        for entry in &system_pack.entries {
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
                // Check if source file was modified since icon was cached, for invalidation
                if let (Some(cached), Some(current)) = (entry.source_mtime, current_mtime) {
                    if cached != current {
                        return false;
                    }
                }

                if let Some(redirect) = &entry.redirect {
                    return Self::_has_app_icon(system_pack, None, Some(redirect));
                }

                if let Some(icon) = &entry.icon {
                    if Self::icon_exists(icon) {
                        return true;
                    }
                }
            };
        }

        false
    }

    /// Get icon pack by app user model id, filename or path
    pub fn has_app_icon(&self, umid: Option<&str>, path: Option<&Path>) -> bool {
        self.with_system_pack(|system_pack| Self::_has_app_icon(system_pack, umid, path))
    }

    pub fn has_shared_file_icon(&self, path: &Path) -> bool {
        let Some(ext) = path.extension() else {
            return false;
        };
        let extension = ext.to_string_lossy().to_lowercase();

        self.with_system_pack(|system_pack| {
            for entry in &system_pack.entries {
                if let IconPackEntry::Shared(entry) = entry {
                    if entry.extension.to_lowercase() == extension && Self::icon_exists(&entry.icon)
                    {
                        return true;
                    }
                }
            }
            false
        })
    }
}

fn last_edit_at(path: &Path) -> Option<DateTime<Utc>> {
    let meta = std::fs::metadata(path).ok()?;
    let date = meta.modified().ok()?;
    Some(date.into())
}
