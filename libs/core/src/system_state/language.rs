#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardLayout {
    /// KLID ex: "00000409" or "0000080a" or "00010409"
    pub id: String,
    /// HKL
    pub handle: String,
    pub display_name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct SystemLanguage {
    pub id: String,
    pub code: String,
    pub name: String,
    pub native_name: String,
    pub input_methods: Vec<KeyboardLayout>,
}
