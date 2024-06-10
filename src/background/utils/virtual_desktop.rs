use color_eyre::eyre::eyre;
use windows::{core::GUID, Win32::Foundation::HWND};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

use crate::{error_handler::Result, windows_api::WindowsApi};

pub struct VirtualDesktopManager {}

pub struct VirtualDesktop {
    #[allow(dead_code)]
    id: [u8; 16],
    guid: GUID,
    #[allow(dead_code)]
    name: String,
}

impl From<GUID> for VirtualDesktop {
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

impl From<Vec<u8>> for VirtualDesktop {
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

impl VirtualDesktop {
    pub fn id(&self) -> String {
        format!("{:?}", self.guid)
    }

    pub fn guid(&self) -> GUID {
        self.guid
    }
}

impl VirtualDesktopManager {
    pub fn enum_virtual_desktops() -> Result<Vec<VirtualDesktop>> {
        let session_id = WindowsApi::current_session_id()?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // This is the path on Windows 10
        let mut current = hkcu
        .open_subkey(format!(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SessionInfo\{session_id}\VirtualDesktops"
        ))
        .ok()
        .and_then(
            |desktops| match desktops.get_raw_value("VirtualDesktopIDs") {
                Ok(current) => Option::from(current.bytes),
                Err(_) => None,
            },
        );

        // This is the path on Windows 11
        if current.is_none() {
            current = hkcu
                .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops")
                .ok()
                .and_then(
                    |desktops| match desktops.get_raw_value("VirtualDesktopIDs") {
                        Ok(current) => Option::from(current.bytes),
                        Err(_) => None,
                    },
                );
        }

        match current {
            Some(current) => {
                let mut result = Vec::new();
                for desktop_id in current.chunks_exact(16).map(|chunk| Vec::from(chunk)) {
                    result.push(VirtualDesktop::from(desktop_id))
                }
                Ok(result)
            }
            None => Err(eyre!("could not determine current virtual desktop").into()),
        }
    }

    pub fn get_by_window(hwnd: HWND) -> Result<VirtualDesktop> {
        Ok(VirtualDesktop::from(
            winvd::get_desktop_by_window(hwnd)?.get_id()?,
        ))
    }

    pub fn get_current_virtual_desktop() -> Result<VirtualDesktop> {
        Ok(VirtualDesktop::from(
            winvd::get_current_desktop()?.get_id()?,
        ))
    }
}
