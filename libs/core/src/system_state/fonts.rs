use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct SeelenFont {
    pub family: String,
}
