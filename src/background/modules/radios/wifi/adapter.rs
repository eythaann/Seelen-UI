use std::collections::{HashMap, HashSet};

use seelen_core::system_state::WlanBssEntry;
use windows::{
    Devices::WiFi::WiFiAdapter,
    Foundation::TypedEventHandler,
    Networking::Connectivity::NetworkInformation,
    Win32::{
        Foundation::HANDLE,
        NetworkManagement::WiFi::{
            DOT11_AUTH_ALGO_80211_OPEN, DOT11_AUTH_ALGO_80211_SHARED_KEY, DOT11_AUTH_ALGO_OWE,
            DOT11_AUTH_ALGO_RSNA, DOT11_AUTH_ALGO_RSNA_PSK, DOT11_AUTH_ALGO_WPA,
            DOT11_AUTH_ALGO_WPA_NONE, DOT11_AUTH_ALGO_WPA_PSK, DOT11_AUTH_ALGO_WPA3,
            DOT11_AUTH_ALGO_WPA3_ENT, DOT11_AUTH_ALGO_WPA3_SAE, DOT11_AUTH_ALGORITHM,
            DOT11_CIPHER_ALGO_BIP, DOT11_CIPHER_ALGO_BIP_CMAC_256, DOT11_CIPHER_ALGO_BIP_GMAC_128,
            DOT11_CIPHER_ALGO_BIP_GMAC_256, DOT11_CIPHER_ALGO_CCMP, DOT11_CIPHER_ALGO_CCMP_256,
            DOT11_CIPHER_ALGO_GCMP, DOT11_CIPHER_ALGO_GCMP_256, DOT11_CIPHER_ALGO_NONE,
            DOT11_CIPHER_ALGO_TKIP, DOT11_CIPHER_ALGO_WEP, DOT11_CIPHER_ALGO_WEP40,
            DOT11_CIPHER_ALGO_WEP104, DOT11_CIPHER_ALGORITHM, DOT11_PHY_TYPE, DOT11_SSID,
            WLAN_AVAILABLE_NETWORK_LIST, WLAN_BSS_ENTRY, WLAN_BSS_LIST, WlanFreeMemory,
            WlanGetAvailableNetworkList, WlanGetNetworkBssList, dot11_BSS_type_any,
            dot11_phy_type_dmg, dot11_phy_type_dsss, dot11_phy_type_eht, dot11_phy_type_erp,
            dot11_phy_type_fhss, dot11_phy_type_he, dot11_phy_type_hrdsss, dot11_phy_type_ht,
            dot11_phy_type_irbaseband, dot11_phy_type_ofdm, dot11_phy_type_vht,
        },
    },
};
use windows_core::HSTRING;

use crate::error::{Result, ResultLogExt};

use super::{WifiManager, WifiManagerEvent};

// ── Known profiles ────────────────────────────────────────────────────────────

pub fn wifi_known_profiles() -> HashSet<String> {
    let mut known = HashSet::new();

    let Ok(profiles) = NetworkInformation::GetConnectionProfiles() else {
        return known;
    };

    for i in 0..profiles.Size().unwrap_or(0) {
        let Ok(profile) = profiles.GetAt(i) else {
            continue;
        };
        if !profile.IsWlanConnectionProfile().unwrap_or(false) {
            continue;
        }
        let Ok(name) = profile.ProfileName() else {
            continue;
        };
        let ssid = name.to_string();
        if !ssid.is_empty() {
            known.insert(ssid);
        }
    }

    known
}

// ── Auth / cipher / PHY labels ──────────────────────────────────────────────

pub fn auth_type_label(auth: DOT11_AUTH_ALGORITHM) -> &'static str {
    match auth {
        DOT11_AUTH_ALGO_80211_OPEN => "Open",
        DOT11_AUTH_ALGO_80211_SHARED_KEY => "WEP",
        DOT11_AUTH_ALGO_WPA => "WPA-Enterprise",
        DOT11_AUTH_ALGO_WPA_PSK => "WPA-Personal",
        DOT11_AUTH_ALGO_WPA_NONE => "WPA-None",
        DOT11_AUTH_ALGO_RSNA => "WPA2-Enterprise",
        DOT11_AUTH_ALGO_RSNA_PSK => "WPA2-Personal",
        DOT11_AUTH_ALGO_WPA3 => "WPA3-Enterprise",
        DOT11_AUTH_ALGO_WPA3_SAE => "WPA3-Personal",
        DOT11_AUTH_ALGO_OWE => "Enhanced Open",
        DOT11_AUTH_ALGO_WPA3_ENT => "WPA3-Enterprise-192",
        _ => "Unknown",
    }
}

pub fn cipher_algo_label(cipher: DOT11_CIPHER_ALGORITHM) -> &'static str {
    match cipher {
        DOT11_CIPHER_ALGO_NONE => "None",
        DOT11_CIPHER_ALGO_WEP40 => "WEP40",
        DOT11_CIPHER_ALGO_TKIP => "TKIP",
        DOT11_CIPHER_ALGO_WEP104 => "WEP104",
        DOT11_CIPHER_ALGO_WEP => "WEP",
        DOT11_CIPHER_ALGO_CCMP => "AES-CCMP",
        DOT11_CIPHER_ALGO_CCMP_256 => "AES-CCMP-256",
        DOT11_CIPHER_ALGO_GCMP => "AES-GCMP",
        DOT11_CIPHER_ALGO_GCMP_256 => "AES-GCMP-256",
        DOT11_CIPHER_ALGO_BIP => "BIP",
        DOT11_CIPHER_ALGO_BIP_GMAC_128 => "BIP-GMAC-128",
        DOT11_CIPHER_ALGO_BIP_GMAC_256 => "BIP-GMAC-256",
        DOT11_CIPHER_ALGO_BIP_CMAC_256 => "BIP-CMAC-256",
        _ => "Unknown",
    }
}

#[allow(non_upper_case_globals)]
pub fn phy_type_label(phy: DOT11_PHY_TYPE) -> &'static str {
    match phy {
        dot11_phy_type_fhss => "802.11 FHSS",
        dot11_phy_type_dsss => "802.11 DSSS",
        dot11_phy_type_irbaseband => "802.11 IR Baseband",
        dot11_phy_type_ofdm => "802.11a",
        dot11_phy_type_hrdsss => "802.11b",
        dot11_phy_type_erp => "802.11g",
        dot11_phy_type_ht => "802.11n (Wi-Fi 4)",
        dot11_phy_type_vht => "802.11ac (Wi-Fi 5)",
        dot11_phy_type_he => "802.11ax (Wi-Fi 6)",
        dot11_phy_type_eht => "802.11be (Wi-Fi 7)",
        dot11_phy_type_dmg => "802.11ad (WiGig)",
        _ => "Unknown",
    }
}

/// Derive the frequency band label and 802.11 channel number from a center
/// frequency expressed in kHz.
pub fn band_and_channel(freq_khz: u32) -> (&'static str, u32) {
    let freq_mhz = freq_khz / 1000;
    match freq_mhz {
        2412..=2472 => ("2.4GHz", (freq_mhz - 2407) / 5),
        2484 => ("2.4GHz", 14),
        5150..=5895 => ("5GHz", (freq_mhz - 5000) / 5),
        5935 => ("6GHz", 2),
        5945..=7115 => ("6GHz", (freq_mhz - 5950) / 5),
        58320..=70200 => ("60GHz", (freq_mhz - 56160) / 2160 + 1),
        _ => ("Unknown", 0),
    }
}

// ── Channel width (parsed from raw 802.11 information elements) ────────────

// Element IDs, per IEEE Std 802.11-2020/802.11be and confirmed against the
// Linux kernel's ieee80211.h / ieee80211-{ht,vht,he,eht}.h headers.
const IE_HT_OPERATION: u8 = 61;
const IE_VHT_OPERATION: u8 = 192;
const IE_EXTENSION: u8 = 255;
const IE_EXT_HE_OPERATION: u8 = 36;
const IE_EXT_EHT_OPERATION: u8 = 106;

/// Best-effort channel-width parse from the raw beacon/probe-response IEs
/// attached to a `WLAN_BSS_ENTRY`. Every access is bounds-checked; malformed
/// or truncated IE data simply yields `None` instead of panicking.
fn parse_channel_width(ies: &[u8]) -> Option<&'static str> {
    let mut ht_width: Option<&'static str> = None;
    let mut vht_width: Option<&'static str> = None;
    let mut he_width: Option<&'static str> = None;
    let mut eht_width: Option<&'static str> = None;

    let mut i = 0usize;
    while i + 2 <= ies.len() {
        let id = ies[i];
        let len = ies[i + 1] as usize;
        let data_start = i + 2;
        let data_end = data_start + len;
        if data_end > ies.len() {
            break;
        }
        let data = &ies[data_start..data_end];

        match id {
            IE_HT_OPERATION if data.len() >= 2 => {
                // struct ieee80211_ht_operation { primary_chan, ht_param, ... }
                let sco = data[1] & 0x03;
                let width_40_allowed = data[1] & 0x04 != 0;
                ht_width = Some(if width_40_allowed && sco != 0 {
                    "40MHz"
                } else {
                    "20MHz"
                });
            }
            IE_VHT_OPERATION if !data.is_empty() => {
                // struct ieee80211_vht_operation { chan_width, ... }
                vht_width = match data[0] {
                    1 => Some("80MHz"),
                    2 => Some("160MHz"),
                    3 => Some("80+80MHz"),
                    // 0 means "follow the HT element", handled by falling back to ht_width below
                    _ => None,
                };
            }
            IE_EXTENSION if !data.is_empty() && data[0] == IE_EXT_HE_OPERATION => {
                he_width = parse_he_operation_width(&data[1..]);
            }
            IE_EXTENSION if !data.is_empty() && data[0] == IE_EXT_EHT_OPERATION => {
                eht_width = parse_eht_operation_width(&data[1..]);
            }
            _ => {}
        }

        i = data_end;
    }

    eht_width.or(he_width).or(vht_width).or(ht_width)
}

/// Parses the optional "6 GHz Operation Information" subfield of an HE
/// Operation element (`struct ieee80211_he_operation` in the Linux kernel)
/// to recover the channel width on the 6 GHz band, where legacy HT/VHT
/// elements are not present. Returns `None` on any layout that doesn't match
/// the expected shape rather than guessing.
fn parse_he_operation_width(params: &[u8]) -> Option<&'static str> {
    // he_oper_params (4 bytes, only the low 24 bits carry flags we need) +
    // he_mcs_nss_set (2 bytes) = 6 fixed bytes before the optional fields.
    if params.len() < 6 {
        return None;
    }
    let he_oper_params = u32::from_le_bytes([params[0], params[1], params[2], 0]);
    const VHT_OPER_INFO_PRESENT: u32 = 1 << 14; // IEEE80211_HE_OPERATION_VHT_OPER_INFO
    const CO_HOSTED_BSS_PRESENT: u32 = 1 << 15; // IEEE80211_HE_OPERATION_CO_HOSTED_BSS
    const SIX_GHZ_OP_INFO_PRESENT: u32 = 1 << 17; // IEEE80211_HE_OPERATION_6GHZ_OP_INFO

    let mut offset = 6usize;
    if he_oper_params & VHT_OPER_INFO_PRESENT != 0 {
        offset += 3;
    }
    if he_oper_params & CO_HOSTED_BSS_PRESENT != 0 {
        offset += 1;
    }
    if he_oper_params & SIX_GHZ_OP_INFO_PRESENT != 0 {
        // struct ieee80211_he_6ghz_oper { primary, control, ccfs0, ccfs1, minrate }
        let control = *params.get(offset + 1)?;
        return match control & 0x03 {
            0 => Some("20MHz"),
            1 => Some("40MHz"),
            2 => Some("80MHz"),
            3 => Some("160MHz"),
            _ => None,
        };
    }
    None
}

/// Parses the optional "EHT Operation Information" subfield of an EHT
/// Operation element (`struct ieee80211_eht_operation` in the Linux kernel,
/// P802.11be section 9.4.2.311) to recover the channel width, including the
/// 320 MHz width introduced by Wi-Fi 7. Returns `None` on any layout that
/// doesn't match the expected shape rather than guessing.
fn parse_eht_operation_width(data: &[u8]) -> Option<&'static str> {
    // params (1 byte) + Basic EHT-MCS/NSS Set (4 fixed bytes) precede the
    // optional EHT Operation Information subfield.
    let params = *data.first()?;
    const INFO_PRESENT: u8 = 0x01; // IEEE80211_EHT_OPER_INFO_PRESENT
    if params & INFO_PRESENT == 0 {
        return None;
    }
    // struct ieee80211_eht_operation_info { control, ccfs0, ccfs1, ... }
    let control = *data.get(5)?;
    match control & 0x07 {
        0 => Some("20MHz"),
        1 => Some("40MHz"),
        2 => Some("80MHz"),
        3 => Some("160MHz"),
        4 => Some("320MHz"),
        _ => None,
    }
}

/// Reads the beacon IE blob attached to `bss` and derives its channel width.
/// `list_start`/`list_end` bound the single allocation returned by
/// `WlanGetNetworkBssList`; if the reported offset/size would read outside of
/// it, this bails out with `None` instead of dereferencing unchecked memory.
unsafe fn bss_channel_width(
    bss: &WLAN_BSS_ENTRY,
    list_start: usize,
    list_end: usize,
) -> Option<&'static str> {
    let ie_size = bss.ulIeSize as usize;
    if ie_size == 0 || ie_size > 4096 {
        return None;
    }
    let entry_addr = bss as *const WLAN_BSS_ENTRY as usize;
    let ie_start = entry_addr.checked_add(bss.ulIeOffset as usize)?;
    let ie_end = ie_start.checked_add(ie_size)?;
    if ie_start < list_start || ie_end > list_end {
        return None;
    }
    let ie_bytes = unsafe { std::slice::from_raw_parts(ie_start as *const u8, ie_size) };
    parse_channel_width(ie_bytes)
}

fn dot11_ssid_to_string(ssid: &DOT11_SSID) -> String {
    let len = (ssid.uSSIDLength as usize).min(ssid.ucSSID.len());
    String::from_utf8_lossy(&ssid.ucSSID[..len]).into_owned()
}

/// Query the native WLAN BSS list for `iface_guid`, enriching each physical
/// access point with the security summary reported for its SSID.
pub fn scan_bss_list(
    handle: HANDLE,
    iface_guid: &windows_core::GUID,
    known: &HashSet<String>,
    connected: &HashSet<String>,
) -> Result<Vec<WlanBssEntry>> {
    unsafe {
        let mut net_info: HashMap<String, (bool, DOT11_AUTH_ALGORITHM, DOT11_CIPHER_ALGORITHM)> =
            HashMap::new();
        let mut net_list_ptr: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
        let err = WlanGetAvailableNetworkList(handle, iface_guid, 0, None, &mut net_list_ptr);
        if err == 0 && !net_list_ptr.is_null() {
            let list = &*net_list_ptr;
            let nets =
                std::slice::from_raw_parts(list.Network.as_ptr(), list.dwNumberOfItems as usize);
            for n in nets {
                net_info.insert(
                    dot11_ssid_to_string(&n.dot11Ssid),
                    (
                        n.bSecurityEnabled.as_bool(),
                        n.dot11DefaultAuthAlgorithm,
                        n.dot11DefaultCipherAlgorithm,
                    ),
                );
            }
            WlanFreeMemory(net_list_ptr as *const _);
        }

        let mut bss_list_ptr: *mut WLAN_BSS_LIST = std::ptr::null_mut();
        let err = WlanGetNetworkBssList(
            handle,
            iface_guid,
            None,
            dot11_BSS_type_any,
            false,
            None,
            &mut bss_list_ptr,
        );
        if err != 0 || bss_list_ptr.is_null() {
            return Err(format!("WlanGetNetworkBssList failed: {err}").into());
        }
        let bss_list = &*bss_list_ptr;
        let bss_entries = std::slice::from_raw_parts(
            bss_list.wlanBssEntries.as_ptr(),
            bss_list.dwNumberOfItems as usize,
        );
        let list_start = bss_list_ptr as usize;
        // A bogus/overflowing dwTotalSize collapses the valid range to empty
        // instead of wrapping, so every bss_channel_width bounds check below
        // safely rejects it.
        let list_end = list_start
            .checked_add(bss_list.dwTotalSize as usize)
            .unwrap_or(list_start);

        let mut out = Vec::with_capacity(bss_entries.len());
        for bss in bss_entries {
            let ssid_raw = dot11_ssid_to_string(&bss.dot11Ssid);
            let ssid = if ssid_raw.is_empty() {
                None
            } else {
                Some(ssid_raw.clone())
            };
            let bssid = format!(
                "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                bss.dot11Bssid[0],
                bss.dot11Bssid[1],
                bss.dot11Bssid[2],
                bss.dot11Bssid[3],
                bss.dot11Bssid[4],
                bss.dot11Bssid[5],
            );
            let (secured, auth, cipher) = net_info
                .get(&ssid_raw)
                .map(|(s, a, c)| (*s, Some(*a), Some(*c)))
                .unwrap_or((false, None, None));
            let (band, channel) = band_and_channel(bss.ulChCenterFrequency);
            let bandwidth = bss_channel_width(bss, list_start, list_end).map(str::to_string);

            out.push(WlanBssEntry {
                known: ssid.as_deref().is_some_and(|s| known.contains(s)),
                connected: ssid.as_deref().is_some_and(|s| connected.contains(s)),
                ssid,
                bssid,
                channel_frequency: bss.ulChCenterFrequency,
                channel,
                band: band.to_string(),
                signal: bss.uLinkQuality,
                rssi: bss.lRssi,
                phy_type: phy_type_label(bss.dot11BssPhyType).to_string(),
                bandwidth,
                secured,
                auth: auth.map(auth_type_label).unwrap_or("Unknown").to_string(),
                cipher: cipher.map(|c| cipher_algo_label(c).to_string()),
            });
        }
        WlanFreeMemory(bss_list_ptr as *const _);
        Ok(out)
    }
}

// ── SluWifiAdapter ────────────────────────────────────────────────────────────

/// RAII wrapper around a single `WiFiAdapter`.
///
/// Subscribes to `AvailableNetworksChanged` on construction and unregisters on drop.
/// When the event fires it sends `WifiManagerEvent::NetworksChanged` so the
/// infrastructure layer can emit updated network data to webviews.
pub struct SluWifiAdapter {
    #[allow(dead_code)]
    pub id: String,
    pub raw: WiFiAdapter,
    token: i64,
}

unsafe impl Send for SluWifiAdapter {}

impl SluWifiAdapter {
    /// Obtain the adapter by device ID and subscribe to network change events.
    pub fn create(device_id: &str) -> Result<Self> {
        let raw = WiFiAdapter::FromIdAsync(&HSTRING::from(device_id))?.join()?;

        let token = raw.AvailableNetworksChanged(&TypedEventHandler::new(|_, _| {
            WifiManager::send(WifiManagerEvent::NetworksChanged);
            Ok(())
        }))?;

        Ok(Self {
            id: device_id.to_string(),
            raw,
            token,
        })
    }

    /// Trigger a hardware scan. Rate-limited calls (E_ABORT) are silently ignored.
    pub fn scan(&self) {
        let adapter = self.raw.clone();
        std::thread::spawn(move || {
            if let Ok(operation) = adapter.ScanAsync()
                && operation.join().is_ok()
            {
                WifiManager::send(WifiManagerEvent::NetworksChanged);
            }
        });
    }

    /// SSID of the network this adapter is currently connected to, if any.
    pub fn get_connected_ssid(&self) -> Option<String> {
        let profile = self
            .raw
            .NetworkAdapter()
            .ok()?
            .GetConnectedProfileAsync()
            .ok()?
            .join()
            .ok()?;
        profile
            .ProfileName()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
    }
}

impl Drop for SluWifiAdapter {
    fn drop(&mut self) {
        self.raw
            .RemoveAvailableNetworksChanged(self.token)
            .log_error();
    }
}
