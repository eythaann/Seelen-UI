use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use itertools::Itertools;
use windows::{
    core::GUID,
    Win32::{
        Foundation::HANDLE,
        NetworkManagement::WiFi::{
            dot11_BSS_type_any, wlan_interface_state_connected,
            wlan_intf_opcode_current_connection, WlanCloseHandle, WlanEnumInterfaces,
            WlanGetNetworkBssList, WlanOpenHandle, WlanQueryInterface, WlanScan,
            WLAN_API_VERSION_2_0, WLAN_BSS_ENTRY, WLAN_BSS_LIST, WLAN_CONNECTION_ATTRIBUTES,
            WLAN_INTERFACE_INFO_LIST,
        },
    },
};

use crate::{error_handler::Result, modules::network::domain::WlanBssEntry};

use super::NetworkManager;

impl From<&WLAN_BSS_ENTRY> for WlanBssEntry {
    fn from(entry: &WLAN_BSS_ENTRY) -> Self {
        let ssid = String::from_utf8_lossy(&entry.dot11Ssid.ucSSID)
            .replace("\0", "")
            .to_string();

        let ssid = if ssid.is_empty() { None } else { Some(ssid) };

        let bssid = entry
            .dot11Bssid
            .iter()
            .map(|b| format!("{:02x}", b))
            .join(":");

        Self {
            ssid,
            bssid,
            channel_frequency: entry.ulChCenterFrequency,
            signal: entry.uLinkQuality,
            connected: false,
            connected_channel: false,
        }
    }
}

static SCANNING: AtomicBool = AtomicBool::new(false);

impl NetworkManager {
    fn open_wlan() -> Result<HANDLE> {
        let mut client_handle = HANDLE::default();
        let mut negotiated_version = 0;

        let result = unsafe {
            WlanOpenHandle(
                WLAN_API_VERSION_2_0,
                None,
                &mut negotiated_version,
                &mut client_handle,
            )
        };

        if result != 0 {
            return Err(format!("Failed to open Wlan, error code: {}", result).into());
        }

        Ok(client_handle)
    }

    fn get_connected_wlan<'a>(
        handle: HANDLE,
        guid: &GUID,
    ) -> Option<&'a WLAN_CONNECTION_ATTRIBUTES> {
        let mut connection_ptr = std::ptr::null_mut::<WLAN_CONNECTION_ATTRIBUTES>();
        let mut data_size = 0;
        unsafe {
            WlanQueryInterface(
                handle,
                guid,
                wlan_intf_opcode_current_connection,
                None,
                &mut data_size,
                &mut connection_ptr as *mut _ as _,
                None,
            );

            if connection_ptr.is_null() {
                None
            } else {
                Some(&*connection_ptr)
            }
        }
    }

    pub fn is_connected_to(ssid: &str) -> Result<bool> {
        let client_handle = Self::open_wlan()?;
        unsafe {
            let mut interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
            let result = WlanEnumInterfaces(client_handle, None, &mut interface_list_ptr);

            if result != 0 || interface_list_ptr.is_null() {
                return Err(format!("Failed to get interface list, error code: {}", result).into());
            }

            let interface_list = &*interface_list_ptr;
            let interfaces = std::slice::from_raw_parts(
                interface_list.InterfaceInfo.as_ptr(),
                interface_list.dwNumberOfItems as usize,
            );

            for interface in interfaces {
                let connection = Self::get_connected_wlan(client_handle, &interface.InterfaceGuid);

                if let Some(connection) = connection {
                    let connected_ssid = String::from_utf8_lossy(
                        &connection.wlanAssociationAttributes.dot11Ssid.ucSSID,
                    )
                    .replace("\0", "")
                    .to_string();

                    if connected_ssid == ssid {
                        return Ok(connection.isState.0 & wlan_interface_state_connected.0
                            == connection.isState.0);
                    }
                }
            }

            WlanCloseHandle(client_handle, None);
        }

        Ok(false)
    }

    pub fn scan_networks() -> Result<Vec<WlanBssEntry>> {
        let client_handle = Self::open_wlan()?;
        let mut wlan_entries = Vec::new();

        unsafe {
            let mut interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
            let result = WlanEnumInterfaces(client_handle, None, &mut interface_list_ptr);

            if result != 0 || interface_list_ptr.is_null() {
                return Err(format!("Failed to get interface list, error code: {}", result).into());
            }

            let interface_list = &*interface_list_ptr;
            let interfaces = std::slice::from_raw_parts(
                interface_list.InterfaceInfo.as_ptr(),
                interface_list.dwNumberOfItems as usize,
            );

            for interface in interfaces {
                let interface_guid = interface.InterfaceGuid;
                let result = WlanScan(client_handle, &interface_guid, None, None, None);

                if result != 0 {
                    return Err(format!("Failed to scan, error code: {}", result).into());
                }

                let mut bss_list_ptr = std::ptr::null_mut::<WLAN_BSS_LIST>();
                let result = WlanGetNetworkBssList(
                    client_handle,
                    &interface_guid,
                    None,
                    dot11_BSS_type_any,
                    true,
                    None,
                    &mut bss_list_ptr,
                );

                if result != 0 || bss_list_ptr.is_null() {
                    return Err(format!("Failed to get bss list, error code: {}", result).into());
                }

                let bss_list = &*bss_list_ptr;
                if bss_list.dwNumberOfItems == 0 {
                    continue;
                }

                let connection = Self::get_connected_wlan(client_handle, &interface_guid);
                let is_connected = match connection {
                    Some(connection) => {
                        connection.isState.0 & wlan_interface_state_connected.0
                            == connection.isState.0
                    }
                    None => false,
                };

                let entries = std::slice::from_raw_parts(
                    bss_list.wlanBssEntries.as_ptr(),
                    bss_list.dwNumberOfItems as usize,
                );

                for entry in entries {
                    let mut wrapped_entry = WlanBssEntry::from(entry);

                    if let Some(connection) = connection {
                        if connection.wlanAssociationAttributes.dot11Ssid.ucSSID
                            == entry.dot11Ssid.ucSSID
                        {
                            wrapped_entry.connected = is_connected;
                        }

                        if connection.wlanAssociationAttributes.dot11Bssid == entry.dot11Bssid {
                            wrapped_entry.connected_channel = is_connected;
                        }
                    }

                    wlan_entries.push(wrapped_entry);
                }
            }

            WlanCloseHandle(client_handle, None);
        }

        Ok(wlan_entries)
    }

    pub fn start_scanning<F>(cb: F)
    where
        F: Fn(Vec<WlanBssEntry>) + Send + 'static,
    {
        SCANNING.store(true, Ordering::SeqCst);
        std::thread::spawn(move || {
            let mut attempts = 0;
            loop {
                if !SCANNING.load(Ordering::SeqCst) {
                    break;
                }

                match Self::scan_networks() {
                    Ok(entries) => {
                        // sometimes we get an empty list, because the wlan list is updating after the scan
                        // so we will wait ~10 seconds before show empty list
                        if !entries.is_empty() || attempts > 3 {
                            cb(entries);
                        } else {
                            attempts += 1;
                        }
                    }
                    Err(err) => {
                        log::error!("{}", err);
                    }
                }

                std::thread::sleep(Duration::from_secs(3));
            }
        });
    }

    pub fn stop_scanning() {
        SCANNING.store(false, Ordering::SeqCst);
    }
}
