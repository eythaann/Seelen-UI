use color_eyre::eyre::eyre;
use windows::core::GUID;
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
}

impl VirtualDesktopManager {
    /* pub fn enum_virtual_desktops(&self) -> Result<Vec<GUID>> {
        let mut desktops = vec![];
        let session_id = WindowsApi::current_session_id()?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // This is the path on Windows 10
        let mut current = hkcu
        .open_subkey(format!(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SessionInfo\{session_id}\VirtualDesktops"
        ))
        .ok()
        .and_then(
            |desktops| match desktops.get_raw_value("CurrentVirtualDesktop") {
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
                    |desktops| match desktops.get_raw_value("CurrentVirtualDesktop") {
                        Ok(current) => Option::from(current.bytes),
                        Err(_) => None,
                    },
                );
        }

        match current {
            Some(current) => Ok(GUID::from_u128(u128::from_be_bytes(
                current.as_slice().try_into().expect("invalid length"),
            ))),
            None => Err(eyre!("could not determine current virtual desktop").into()),
        }
    } */

    pub fn get_current_virtual_desktop() -> Result<VirtualDesktop> {
        let session_id = WindowsApi::current_session_id()?;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // This is the path on Windows 10
        let mut current = hkcu
        .open_subkey(format!(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SessionInfo\{session_id}\VirtualDesktops"
        ))
        .ok()
        .and_then(
            |desktops| match desktops.get_raw_value("CurrentVirtualDesktop") {
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
                    |desktops| match desktops.get_raw_value("CurrentVirtualDesktop") {
                        Ok(current) => Option::from(current.bytes),
                        Err(_) => None,
                    },
                );
        }

        match current {
            Some(current) => Ok(VirtualDesktop::from(current)),
            None => Err(eyre!("could not determine current virtual desktop").into()),
        }
    }
}
