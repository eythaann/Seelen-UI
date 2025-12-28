/// Represents a radio device like a bluetooth - wifi etc.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct RadioDevice {
    pub id: String,
    pub name: String,
    pub kind: RadioDeviceKind,
    /// True if the radio device is currently `On`.
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum RadioDeviceKind {
    Other,
    WiFi,
    MobileBroadband,
    Bluetooth,
    FM,
}
