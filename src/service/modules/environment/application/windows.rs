use winreg::{
    RegKey, RegValue,
    enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS, REG_EXPAND_SZ},
};

use crate::{
    error::Result, modules::environment::domain::EnvironmentSystem, windows_api::WindowsApi,
};

const MACHINE_ENVIRONMENT_KEY: &str =
    r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

pub struct WindowsEnvironment;

impl EnvironmentSystem for WindowsEnvironment {
    const PATH_SEPARATOR: char = ';';

    fn create() -> Self {
        Self
    }

    fn is_path_managed_by_os(&self) -> bool {
        WindowsApi::was_installed_using_msix()
    }

    fn system_path(&self) -> Result<String> {
        Ok(open_machine_environment()?.get_value("Path")?)
    }

    // set_value() always writes REG_SZ, which prevents Windows from expanding
    // %SystemRoot%\system32 and other variable-based entries, corrupting the PATH.
    // The system PATH must be REG_EXPAND_SZ so Windows expands variables correctly.
    fn set_system_path(&self, value: &str) -> Result<()> {
        let bytes = value
            .encode_utf16()
            .chain(std::iter::once(0u16))
            .flat_map(|c| c.to_le_bytes())
            .collect();
        open_machine_environment()?.set_raw_value(
            "Path",
            &RegValue {
                bytes,
                vtype: REG_EXPAND_SZ,
            },
        )?;
        Ok(())
    }
}

fn open_machine_environment() -> Result<RegKey> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    Ok(hklm.open_subkey_with_flags(MACHINE_ENVIRONMENT_KEY, KEY_ALL_ACCESS)?)
}
