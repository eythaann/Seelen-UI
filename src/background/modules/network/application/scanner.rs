use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use itertools::Itertools;
use seelen_core::system_state::WlanBssEntry;
use windows::{
    core::GUID,
    Win32::{
        Foundation::HANDLE,
        NetworkManagement::WiFi::{
            dot11_BSS_type_any, wlan_interface_state_connected,
            wlan_intf_opcode_current_connection, WlanCloseHandle, WlanEnumInterfaces,
            WlanGetAvailableNetworkList2, WlanGetNetworkBssList, WlanGetProfile,
            WlanGetProfileList, WlanOpenHandle, WlanQueryInterface, WlanScan,
            DOT11_CAPABILITY_INFO_PRIVACY, WLAN_API_VERSION_2_0,
            WLAN_AVAILABLE_NETWORK_INCLUDE_ALL_ADHOC_PROFILES,
            WLAN_AVAILABLE_NETWORK_INCLUDE_ALL_MANUAL_HIDDEN_PROFILES,
            WLAN_AVAILABLE_NETWORK_LIST_V2, WLAN_AVAILABLE_NETWORK_V2, WLAN_BSS_ENTRY,
            WLAN_BSS_LIST, WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO,
            WLAN_INTERFACE_INFO_LIST, WLAN_PROFILE_INFO_LIST,
        },
    },
};
use windows_core::{PCWSTR, PWSTR};

use crate::error_handler::Result;

use super::NetworkManager;

fn from_raw_entry(entry: &WLAN_BSS_ENTRY) -> WlanBssEntry {
    let ssid = String::from_utf8_lossy(&entry.dot11Ssid.ucSSID)
        .replace("\0", "")
        .to_string();

    let ssid = if ssid.is_empty() { None } else { Some(ssid) };

    let bssid = entry
        .dot11Bssid
        .iter()
        .map(|b| format!("{b:02x}"))
        .join(":");

    WlanBssEntry {
        ssid,
        bssid,
        channel_frequency: entry.ulChCenterFrequency,
        signal: entry.uLinkQuality,
        connected: false,
        connected_channel: false,
        secured: entry.usCapabilityInformation as u32 & DOT11_CAPABILITY_INFO_PRIVACY
            == DOT11_CAPABILITY_INFO_PRIVACY,
        known: false,
    }
}
static SCANNING: AtomicBool = AtomicBool::new(false);

impl NetworkManager {
    pub fn open_wlan() -> Result<HANDLE> {
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
            return Err(format!("Failed to open Wlan, error code: {result}").into());
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
            for interface in Self::get_wlan_interfaces(client_handle)? {
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

    #[allow(dead_code)]
    fn get_profiles(client_handle: HANDLE, interface_guid: &GUID) -> Result<()> {
        unsafe {
            let mut profile_list_ptr = std::ptr::null_mut::<WLAN_PROFILE_INFO_LIST>();
            let result =
                WlanGetProfileList(client_handle, interface_guid, None, &mut profile_list_ptr);
            if result != 0 || profile_list_ptr.is_null() {
                return Err(format!("Failed to get profile list, error code: {result}").into());
            }

            let profile_list = &*profile_list_ptr;
            let entries = std::slice::from_raw_parts(
                profile_list.ProfileInfo.as_ptr(),
                profile_list.dwNumberOfItems as usize,
            );

            for entry in entries {
                let profile_name = PCWSTR(entry.strProfileName.as_ptr());
                let mut profile_xml = PWSTR::null();
                let result = WlanGetProfile(
                    client_handle,
                    interface_guid,
                    profile_name,
                    None,
                    &mut profile_xml,
                    None,
                    None,
                );

                if result != 0 {
                    return Err(format!("Failed to get profile, error code: {result}").into());
                }

                if !profile_xml.is_null() {
                    let profile: serde_json::Value =
                        quick_xml::de::from_str(&profile_xml.to_string()?)?;
                    println!("{profile:#?}")
                }
            }
        }
        Ok(())
    }

    pub fn get_wlan_interfaces(client_handle: HANDLE) -> Result<Vec<WLAN_INTERFACE_INFO>> {
        unsafe {
            let mut interface_list_ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
            let result = WlanEnumInterfaces(client_handle, None, &mut interface_list_ptr);

            if result != 0 || interface_list_ptr.is_null() {
                return Err(format!("Failed to get interface list, error code: {result}").into());
            }

            let interface_list = &*interface_list_ptr;
            let interfaces = std::slice::from_raw_parts(
                interface_list.InterfaceInfo.as_ptr(),
                interface_list.dwNumberOfItems as usize,
            );
            Ok(interfaces.to_vec())
        }
    }

    fn get_available_networks(
        client_handle: HANDLE,
        interface_guid: &GUID,
    ) -> Result<Vec<WLAN_AVAILABLE_NETWORK_V2>> {
        unsafe {
            let mut network_list_ptr = std::ptr::null_mut::<WLAN_AVAILABLE_NETWORK_LIST_V2>();
            let result = WlanGetAvailableNetworkList2(
                client_handle,
                interface_guid,
                WLAN_AVAILABLE_NETWORK_INCLUDE_ALL_ADHOC_PROFILES
                    & WLAN_AVAILABLE_NETWORK_INCLUDE_ALL_MANUAL_HIDDEN_PROFILES,
                None,
                &mut network_list_ptr,
            );

            if result != 0 || network_list_ptr.is_null() {
                return Err(format!("Failed to get network list, error code: {result}").into());
            }

            let network_list = &*network_list_ptr;
            let entries = std::slice::from_raw_parts(
                network_list.Network.as_ptr(),
                network_list.dwNumberOfItems as usize,
            );
            Ok(entries.to_vec())
        }
    }

    fn get_bss_entries(
        client_handle: HANDLE,
        interface_guid: &GUID,
    ) -> Result<Vec<WLAN_BSS_ENTRY>> {
        unsafe {
            let mut bss_list_ptr = std::ptr::null_mut::<WLAN_BSS_LIST>();
            let result = WlanGetNetworkBssList(
                client_handle,
                interface_guid,
                None,
                dot11_BSS_type_any,
                true,
                None,
                &mut bss_list_ptr,
            );

            if result != 0 || bss_list_ptr.is_null() {
                return Err(format!("Failed to get bss list, error code: {result}").into());
            }

            let bss_list = &*bss_list_ptr;
            let entries = std::slice::from_raw_parts(
                bss_list.wlanBssEntries.as_ptr(),
                bss_list.dwNumberOfItems as usize,
            );
            Ok(entries.to_vec())
        }
    }

    pub fn scan_networks() -> Result<Vec<WlanBssEntry>> {
        let client_handle = Self::open_wlan()?;
        let mut wlan_entries = Vec::new();

        unsafe {
            for interface in Self::get_wlan_interfaces(client_handle)? {
                let interface_guid = interface.InterfaceGuid;
                let result = WlanScan(client_handle, &interface_guid, None, None, None);

                if result != 0 {
                    return Err(format!("Failed to scan, error code: {result}").into());
                }

                let available_networks =
                    Self::get_available_networks(client_handle, &interface_guid)?;
                if available_networks.is_empty() {
                    continue;
                }

                let bss_entries = Self::get_bss_entries(client_handle, &interface_guid)?;
                if bss_entries.is_empty() {
                    continue;
                }

                let connection = Self::get_connected_wlan(client_handle, &interface_guid);
                for entry in bss_entries {
                    let mut wrapped_entry = from_raw_entry(&entry);

                    if let Some(connection) = connection {
                        if connection.wlanAssociationAttributes.dot11Ssid.ucSSID
                            == entry.dot11Ssid.ucSSID
                        {
                            wrapped_entry.connected = connection.isState.0
                                & wlan_interface_state_connected.0
                                == connection.isState.0;
                            wrapped_entry.connected_channel = wrapped_entry.connected
                                && connection.wlanAssociationAttributes.dot11Bssid
                                    == entry.dot11Bssid;
                        }
                    }

                    if let Some(network) = available_networks
                        .iter()
                        .find(|n| n.dot11Ssid.ucSSID == entry.dot11Ssid.ucSSID)
                    {
                        let profile = PCWSTR::from_raw(network.strProfileName.as_ptr());
                        wrapped_entry.known = !profile.is_null() && !profile.is_empty();
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
                        log::error!("{err}");
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
