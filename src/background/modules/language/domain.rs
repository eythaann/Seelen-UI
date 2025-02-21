use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct KeyboardLayout {
    /// also known as keyboard layout name or KLID ex: "00000409" or "0000080a" or "00010409"
    pub id: String,
    pub display_name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Language {
    pub code: String,
    pub name: String,
    pub input_methods: Vec<KeyboardLayout>,
}
