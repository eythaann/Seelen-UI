use windows::core::GUID;
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

use crate::{error_handler::Result, windows_api::WindowsApi};

use super::is_windows_10;

pub struct RegistryVirtualDesktopManager {}

pub struct RegistryVirtualDesktop {
    #[allow(dead_code)]
    id: [u8; 16],
    guid: GUID,
    #[allow(dead_code)]
    name: String,
}

impl From<GUID> for RegistryVirtualDesktop {
    fn from(guid: GUID) -> Self {
        let mut id: Vec<u8> = Vec::new();
        id.append(&mut guid.data1.to_le_bytes().to_vec());
        id.append(&mut guid.data2.to_le_bytes().to_vec());
        id.append(&mut guid.data3.to_le_bytes().to_vec());
        id.append(&mut guid.data4.to_vec());

        Self {
            id: id.try_into().expect("Invalid id length"),
            guid,
            name: String::new(),
        }
    }
}

impl From<Vec<u8>> for RegistryVirtualDesktop {
    fn from(id: Vec<u8>) -> Self {
        Self {
            id: id.clone().try_into().expect("Invalid id length"),
            guid: GUID {
                data1: u32::from_le_bytes(id[0..4].try_into().unwrap()),
                data2: u16::from_le_bytes(id[4..6].try_into().unwrap()),
                data3: u16::from_le_bytes(id[6..8].try_into().unwrap()),
                data4: id[8..].try_into().unwrap(),
            },
            name: String::new(),
        }
    }
}

impl RegistryVirtualDesktop {
    pub fn id(&self) -> String {
        format!("{:?}", self.guid)
    }

    pub fn guid(&self) -> GUID {
        self.guid
    }
}

impl RegistryVirtualDesktopManager {
    fn get_virtual_desktops_folder() -> Result<RegKey> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        Ok(if is_windows_10() {
            let session_id = WindowsApi::current_session_id()?;
            hkcu.open_subkey(format!(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SessionInfo\{session_id}\VirtualDesktops"))?
        } else {
            hkcu.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops")?
        })
    }

    pub fn enum_virtual_desktops() -> Result<Vec<RegistryVirtualDesktop>> {
        let desktops = Self::get_virtual_desktops_folder()?;
        let current = desktops.get_raw_value("VirtualDesktopIDs")?.bytes;
        let mut result = Vec::new();
        for desktop_id in current.chunks_exact(16).map(Vec::from) {
            result.push(RegistryVirtualDesktop::from(desktop_id))
        }
        Ok(result)
    }

    pub fn current_virtual_desktops() -> Result<RegistryVirtualDesktop> {
        let desktops = Self::get_virtual_desktops_folder()?;
        let current = desktops.get_raw_value("CurrentVirtualDesktop")?;
        Ok(RegistryVirtualDesktop::from(current.bytes))
    }
}
