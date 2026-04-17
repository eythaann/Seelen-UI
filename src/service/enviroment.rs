use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, REG_EXPAND_SZ},
    RegKey, RegValue,
};

use crate::error::Result;

pub fn was_installed_using_msix() -> bool {
    static CACHE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHE.get_or_init(|| {
        std::env::current_exe().is_ok_and(|p| p.with_file_name("AppxManifest.xml").exists())
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

// set_value() always writes REG_SZ, which prevents Windows from expanding
// %SystemRoot%\system32 and other variable-based entries, corrupting the PATH.
// The system PATH must be REG_EXPAND_SZ so Windows expands variables correctly.
fn set_path_value(enviroment: &RegKey, value: String) -> Result<()> {
    let bytes = value
        .encode_utf16()
        .chain(std::iter::once(0u16))
        .flat_map(|c| c.to_le_bytes())
        .collect();
    enviroment.set_raw_value(
        "Path",
        &RegValue {
            bytes,
            vtype: REG_EXPAND_SZ,
        },
    )?;
    Ok(())
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
        set_path_value(&enviroment, paths.join(";"))?;
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
        set_path_value(&enviroment, paths.join(";"))?;
    }
    Ok(())
}
