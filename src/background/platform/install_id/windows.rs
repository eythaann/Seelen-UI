use uuid::Uuid;
use winreg::{
    RegKey,
    enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ},
};

use crate::error::Result;

const REGISTRY_SUBKEY: &str = r"Software\Seelen UI\Analytics";
const INSTALL_ID_VALUE: &str = "InstallId";

/// Stored at `HKCU\Software\Seelen UI\Analytics` → `InstallId` (REG_SZ).
/// This key is NOT declared in the MSIX package manifest, so it survives
/// across reinstalls and package updates.
pub fn get_or_create() -> Result<Uuid> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(key) = hkcu.open_subkey_with_flags(REGISTRY_SUBKEY, KEY_READ)
        && let Ok(id_str) = key.get_value::<String, _>(INSTALL_ID_VALUE)
        && let Ok(id) = Uuid::parse_str(&id_str)
    {
        return Ok(id);
    }

    let new_id = Uuid::new_v4();
    let (key, _) = hkcu.create_subkey_with_flags(REGISTRY_SUBKEY, KEY_ALL_ACCESS)?;
    key.set_value(INSTALL_ID_VALUE, &new_id.to_string())?;
    log::debug!("generated new install_id {new_id}");
    Ok(new_id)
}
