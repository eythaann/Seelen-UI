#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use crate::error::Result;

use super::domain::{EnvironmentSystem, path_with_entry, path_without_entry};

#[cfg(windows)]
type Platform = windows::WindowsEnvironment;

pub struct EnvironmentManager {
    platform: Platform,
}

impl EnvironmentManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<EnvironmentManager> = LazyLock::new(EnvironmentManager::create);
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
        }
    }

    /// add the installation directory to the PATH environment variable
    pub fn add_installation_dir_to_path(&self) -> Result<()> {
        if self.platform.is_path_managed_by_os() {
            return Ok(());
        }
        let paths = self.platform.system_path()?;
        if let Some(paths) = path_with_entry(&paths, Platform::PATH_SEPARATOR, &install_dir()?) {
            log::trace!("Adding installation directory to PATH environment variable");
            self.platform.set_system_path(&paths)?;
        }
        Ok(())
    }

    /// remove the installation directory from the PATH environment variable
    pub fn remove_installation_dir_from_path(&self) -> Result<()> {
        if self.platform.is_path_managed_by_os() {
            return Ok(());
        }
        let paths = self.platform.system_path()?;
        if let Some(paths) = path_without_entry(&paths, Platform::PATH_SEPARATOR, &install_dir()?) {
            log::trace!("Removing installation directory from PATH environment variable");
            self.platform.set_system_path(&paths)?;
        }
        Ok(())
    }
}

fn install_dir() -> Result<String> {
    let exe = std::env::current_exe()?;
    let dir = exe
        .parent()
        .ok_or("Failed to get the installation directory")?;
    Ok(dir.to_string_lossy().to_string())
}
