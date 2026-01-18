use tauri::{path::BaseDirectory, Manager};

use crate::{app::get_app_handle, error::Result, utils::constants::SEELEN_COMMON};

pub struct RestorationAndMigration;

impl RestorationAndMigration {
    // migration of user settings files below v1.8.3
    fn migration_v1_8_3() -> Result<()> {
        let path = get_app_handle().path();
        let data_path = path.app_data_dir()?;

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
        Ok(())
    }

    // migration of user settings files below v2.1.0
    fn migration_v2_1_0() -> Result<()> {
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

    fn migration_v2_3_9() -> Result<()> {
        let handle = get_app_handle();
        let data_path = handle.path().app_data_dir()?;

        let old_soundpacks = data_path.join("sounds");
        if old_soundpacks.exists() {
            std::fs::remove_dir_all(old_soundpacks)?;
        }

        let old_iconpacks = data_path.join("icons");
        if old_iconpacks.exists() {
            std::fs::remove_dir_all(old_iconpacks)?;
        }
        Ok(())
    }

    // remove old generated icon pack (path changed in v2.4.10)
    fn migration_v2_4_10() -> Result<()> {
        let old_path = SEELEN_COMMON.user_icons_path().join("system");
        if old_path.exists() {
            std::fs::remove_dir_all(old_path)?;
        }
        Ok(())
    }

    fn recreate_user_folders() -> Result<()> {
        std::fs::create_dir_all(SEELEN_COMMON.app_temp_dir())?;

        std::fs::create_dir_all(SEELEN_COMMON.user_themes_path())?;
        std::fs::create_dir_all(SEELEN_COMMON.user_icons_path())?;
        std::fs::create_dir_all(SEELEN_COMMON.user_wallpapers_path())?;
        std::fs::create_dir_all(SEELEN_COMMON.user_sounds_path())?;
        std::fs::create_dir_all(SEELEN_COMMON.user_plugins_path())?;
        std::fs::create_dir_all(SEELEN_COMMON.user_widgets_path())?;
        Ok(())
    }

    pub fn run() -> Result<()> {
        Self::recreate_user_folders()?;
        Self::migration_v1_8_3()?;
        Self::migration_v2_1_0()?;
        Self::migration_v2_3_9()?;
        Self::migration_v2_4_10()?;
        Ok(())
    }
}
