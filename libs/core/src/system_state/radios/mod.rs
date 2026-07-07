/// Represents a radio device like a bluetooth - wifi etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct RadioDevice {
    pub id: String,
    pub name: String,
    pub kind: RadioDeviceKind,
    /// True if the radio device is currently `On`.
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum RadioDeviceKind {
    Other,
    WiFi,
    MobileBroadband,
    Bluetooth,
    FM,
}
