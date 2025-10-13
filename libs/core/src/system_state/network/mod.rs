use serde_alias::serde_alias;

#[serde_alias(PascalCase)] // used by pwsh scripts
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WlanProfile {
    pub profile_name: String,
    #[serde(alias = "SSID")]
    pub ssid: String,
    pub authentication: String,
    pub encryption: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WlanBssEntry {
    pub ssid: Option<String>,
    pub bssid: String,
    pub channel_frequency: u32,
    pub signal: u32,
    /// true if the network is a saved profile
    pub known: bool,
    /// true if the network is encrypted like WEP, WPA, or WPA2
    pub secured: bool,
    /// true if the interface is connected to this network
    pub connected: bool,
    /// true if the interface is connected to this network and is using this channel frequency
    pub connected_channel: bool,
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
