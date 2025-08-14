use tauri::{path::BaseDirectory, Manager};

use crate::{
    app::get_app_handle,
    error::Result,
    utils::{constants::SEELEN_COMMON, copy_dir_all},
};

pub struct RestorationAndMigration;

impl RestorationAndMigration {
    pub fn recreate_profiles() -> Result<()> {
        let user_profiles = SEELEN_COMMON.user_profiles_path();
        if user_profiles.is_dir() && std::fs::read_dir(user_profiles)?.next().is_some() {
            return Ok(());
        }

        let bundled_profiles = SEELEN_COMMON.bundled_profiles_path();
        copy_dir_all(bundled_profiles, user_profiles)?;
        Ok(())
    }

    // migration of user settings files below v2.1.0, will be removed in v3.0
    pub fn migrate_old_toolbar_items() -> Result<()> {
        let old_folder = SEELEN_COMMON.user_placeholders_path();
        let old = old_folder.join("custom.yml");
        if old.exists() {
            std::fs::copy(old, SEELEN_COMMON.toolbar_items_path())?;
            std::fs::rename(
                old_folder,
                old_folder.with_file_name("deprecated_placeholders"),
            )?;
        }
        Ok(())
    }

    pub fn migrate_old_folders() -> Result<()> {
        let handle = get_app_handle();
        let data_path = handle.path().app_data_dir()?;

        let old_soundpacks = data_path.join("sounds");
        if old_soundpacks.exists() {
            std::fs::remove_dir_all(old_soundpacks)?;
        }

        let old_iconpacks = data_path.join("icons");
        if old_iconpacks.exists() {
            let renamed = data_path.join("old_iconpacks");
            if renamed.exists() {
                std::fs::remove_dir_all(&renamed)?;
            }
            std::fs::rename(old_iconpacks, renamed)?;
        }
        Ok(())
    }

    pub fn recreate_user_folders() -> Result<()> {
        let path = get_app_handle().path();
        let data_path = path.app_data_dir()?;

        // migration of user settings files below v1.8.3
        let old_path = path.resolve(".config/seelen", BaseDirectory::Home)?;
        if old_path.exists() {
            log::trace!("Migrating user settings from {old_path:?}");
            for entry in std::fs::read_dir(&old_path)?.flatten() {
                if entry.file_type()?.is_dir() {
                    continue;
                }
                std::fs::copy(entry.path(), data_path.join(entry.file_name()))?;
            }
            std::fs::remove_dir_all(&old_path)?;
        }

        // temporal folder to group artifacts
        std::fs::create_dir_all(SEELEN_COMMON.app_temp_dir())?;

        let create_if_needed = move |folder: &str| -> Result<()> {
            let path = data_path.join(folder);
            if !path.exists() {
                std::fs::create_dir_all(path)?;
            }
            Ok(())
        };
        create_if_needed("themes")?;
        create_if_needed("iconpacks/system")?;
        create_if_needed("wallpapers")?;
        create_if_needed("soundpacks")?;
        create_if_needed("plugins")?;
        create_if_needed("widgets")?;
        Self::recreate_profiles()?;

        Ok(())
    }

    pub fn run_full() -> Result<()> {
        Self::recreate_user_folders()?;
        Self::migrate_old_toolbar_items()?;
        Self::migrate_old_folders()?;
        Ok(())
    }
}
