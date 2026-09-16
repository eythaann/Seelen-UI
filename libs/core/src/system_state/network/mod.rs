#[derive(Debug, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct WlanBssEntry {
    /// None for hidden networks (SSID not broadcast)
    pub ssid: Option<String>,
    pub bssid: String,
    /// Channel center frequency in kHz
    pub channel_frequency: u32,
    /// 802.11 channel number derived from `channel_frequency`, 0 if unknown
    pub channel: u32,
    /// Frequency band, e.g. "2.4GHz", "5GHz", "6GHz", "60GHz"
    pub band: String,
    /// Signal quality 0–100 (native WLAN link quality)
    pub signal: u32,
    /// Raw received signal strength in dBm
    pub rssi: i32,
    /// PHY / Wi-Fi generation label, e.g. "802.11ax (Wi-Fi 6)"
    pub phy_type: String,
    /// Channel width parsed from the beacon's HT/VHT/HE information elements,
    /// e.g. "20MHz", "40MHz", "80MHz", "160MHz", "80+80MHz". None if it could
    /// not be determined.
    pub bandwidth: Option<String>,
    /// true if Windows has a saved profile for this network
    pub known: bool,
    /// true if the network requires authentication (WEP/WPA/WPA2/WPA3)
    pub secured: bool,
    /// Human-readable authentication type, e.g. "WPA2-Personal", "Open", "WPA3-Personal", "Enhanced Open"
    pub auth: String,
    /// Human-readable cipher algorithm, e.g. "AES-CCMP", "TKIP", "WEP", None if unknown
    pub cipher: Option<String>,
    /// true if currently connected to this network
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum AdapterStatus {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct NetworkAdapter {
    // General information
    pub name: String,
    pub description: String,
    pub status: AdapterStatus,
    pub dns_suffix: String,
    #[serde(rename = "type")]
    pub interface_type: String,
    // Address information
    pub ipv6: Option<String>,
    pub ipv4: Option<String>,
    pub gateway: Option<String>,
    pub mac: String,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Hotspot {
    pub clients: u32,
    pub max_clients: u32,
    pub state: HotspotState,
    pub ssid: Option<String>,
    pub passphrase: Option<String>,
    pub band: String,
    pub encryption: String,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum HotspotState {
    Unknown,
    On,
    Off,
    InTransition,
}
