use tauri::{path::BaseDirectory, Manager};

use crate::{error_handler::Result, seelen::get_app_handle, utils::copy_dir_all};

pub struct RestorationAndMigration;

impl RestorationAndMigration {
    pub fn recreate_profiles() -> Result<()> {
        let path = get_app_handle().path();
        let user_profiles = path.app_data_dir()?.join("profiles");
        if user_profiles.is_dir() && std::fs::read_dir(&user_profiles)?.next().is_some() {
            return Ok(());
        }

        let bundled_profiles = path.resource_dir()?.join("static/profiles");
        copy_dir_all(bundled_profiles, user_profiles)?;
        Ok(())
    }

    pub fn recreate_user_folders() -> Result<()> {
        let path = get_app_handle().path();
        let data_path = path.app_data_dir()?;

        // migration of user settings files below v1.8.3
        let old_path = path.resolve(".config/seelen", BaseDirectory::Home)?;
        if old_path.exists() {
            log::trace!("Migrating user settings from {:?}", old_path);
            for entry in std::fs::read_dir(&old_path)?.flatten() {
                if entry.file_type()?.is_dir() {
                    continue;
                }
                std::fs::copy(entry.path(), data_path.join(entry.file_name()))?;
            }
            std::fs::remove_dir_all(&old_path)?;
        }

        let create_if_needed = move |folder: &str| -> Result<()> {
            let path = data_path.join(folder);
            if !path.exists() {
                std::fs::create_dir_all(path)?;
            }
            Ok(())
        };

        create_if_needed("themes")?;
        create_if_needed("layouts")?;
        create_if_needed("placeholders")?;
        create_if_needed("icons/system")?;
        create_if_needed("wallpapers")?;
        create_if_needed("plugins")?;
        create_if_needed("widgets")?;
        Self::recreate_profiles()?;

        Ok(())
    }

    pub fn run_full() -> Result<()> {
        Self::recreate_user_folders()?;
        Ok(())
    }
}
