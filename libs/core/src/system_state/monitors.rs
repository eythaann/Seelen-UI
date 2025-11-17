use crate::{identifier_impl, rect::Rect};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalMonitor {
    pub id: MonitorId,
    pub name: String,
    pub rect: Rect,
    pub dpi: f64,
    pub is_primary: bool,
}

#[derive(Debug, Serialize, Deserialize, TS)]
pub struct Brightness {
    pub min: u32,
    pub max: u32,
    pub current: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct MonitorId(pub String);

identifier_impl!(MonitorId, String);
