use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS},
    RegKey,
};

use crate::error::Result;

pub fn was_installed_using_msix() -> bool {
    std::env::current_exe().is_ok_and(|p| {
        p.with_file_name("AppxManifest.xml").exists()
            || p.starts_with("C:\\Program Files\\WindowsApps")
    })
}

pub fn open_machine_enviroment() -> Result<RegKey> {
    let hkcr = RegKey::predef(HKEY_LOCAL_MACHINE);
    let enviroment = hkcr.open_subkey_with_flags(
        r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        KEY_ALL_ACCESS,
    )?;
    Ok(enviroment)
}

/// add the installation directory to the PATH environment variable
pub fn add_installation_dir_to_path() -> Result<()> {
    if was_installed_using_msix() {
        return Ok(());
    }

    let enviroment = open_machine_enviroment()?;
    let env_path: String = enviroment.get_value("Path")?;

    let install_folder = std::env::current_exe()?
        .parent()
        .expect("Failed to get parent directory")
        .to_string_lossy()
        .to_string();
    if !env_path.contains(&install_folder) {
        log::trace!("Adding installation directory to PATH environment variable");
        enviroment.set_value("Path", &format!("{};{}", install_folder, env_path))?;
    }
    Ok(())
}

/// remove the installation directory from the PATH environment variable
pub fn remove_installation_dir_from_path() -> Result<()> {
    if was_installed_using_msix() {
        return Ok(());
    }

    let enviroment = open_machine_enviroment()?;
    let env_path: String = enviroment.get_value("Path")?;

    let install_folder = std::env::current_exe()?
        .parent()
        .expect("Failed to get parent directory")
        .to_string_lossy()
        .to_string();
    if env_path.contains(&install_folder) {
        log::trace!("Removing installation directory from PATH environment variable");
        let new_path = env_path.replace(&format!("{install_folder};"), "");
        enviroment.set_value("Path", &new_path)?;
    }
    Ok(())
}
