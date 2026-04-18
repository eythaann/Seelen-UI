#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WlanBssEntry {
    /// None for hidden networks (SSID not broadcast)
    pub ssid: Option<String>,
    pub bssid: String,
    /// Channel center frequency in kHz
    pub channel_frequency: u32,
    /// Signal strength 0–100 (derived from WinRT SignalBars × 20)
    pub signal: u32,
    /// true if Windows has a saved profile for this network
    pub known: bool,
    /// true if the network requires authentication (WEP/WPA/WPA2/WPA3)
    pub secured: bool,
    /// Human-readable authentication type, e.g. "WPA2-Personal", "Open"
    pub auth: String,
    /// true if currently connected to this network
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(repr(enum = name))]
pub enum AdapterStatus {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, TS)]
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
