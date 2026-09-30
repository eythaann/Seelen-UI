use crate::{identifier_impl, rect::Rect};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "gen-binds", not(feature = "salvo")),
    ts(optional_fields = nullable)
)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalMonitor {
    pub id: MonitorId,
    pub name: String,
    pub rect: Rect,
    pub scale_factor: f64,
    pub is_primary: bool,
    /// `None` when the monitor does not support HDR / advanced color.
    pub hdr: Option<bool>,
    /// Brightness as a percentage (0-100), `None` when the monitor does not support
    /// brightness control.
    pub brightness: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct MonitorId(pub String);

identifier_impl!(MonitorId, String);

impl Default for MonitorId {
    fn default() -> Self {
        Self("null".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum AppBarEdge {
    Top,
    Left,
    Bottom,
    Right,
}
