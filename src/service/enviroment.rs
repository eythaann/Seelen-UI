use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS},
    RegKey,
};

use crate::error::Result;

pub fn was_installed_using_msix() -> bool {
    std::env::current_exe().is_ok_and(|p| {
        p.with_file_name("AppxManifest.xml").exists()
            || p.to_string_lossy()
                .to_lowercase()
                .starts_with("c:\\program files\\windowsapps")
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
    let paths: String = enviroment.get_value("Path")?;
    let mut paths: Vec<String> = paths.split(';').map(|s| s.to_owned()).collect();

    let install_folder = std::env::current_exe()?
        .parent()
        .expect("Failed to get parent directory")
        .to_string_lossy()
        .to_string();

    if !paths.contains(&install_folder) {
        log::trace!("Adding installation directory to PATH environment variable");
        paths.push(install_folder);
        enviroment.set_value("Path", &paths.join(";"))?;
    }
    Ok(())
}

/// remove the installation directory from the PATH environment variable
pub fn remove_installation_dir_from_path() -> Result<()> {
    if was_installed_using_msix() {
        return Ok(());
    }

    let enviroment = open_machine_enviroment()?;
    let paths: String = enviroment.get_value("Path")?;
    let mut paths: Vec<String> = paths.split(';').map(|s| s.to_owned()).collect();

    let install_folder = std::env::current_exe()?
        .parent()
        .expect("Failed to get parent directory")
        .to_string_lossy()
        .to_string();

    if paths.contains(&install_folder) {
        log::trace!("Removing installation directory from PATH environment variable");
        paths.retain(|p| p != &install_folder);
        enviroment.set_value("Path", &paths.join(";"))?;
    }
    Ok(())
}
