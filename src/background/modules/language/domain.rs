use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct KeyboardLayout {
    pub id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Language {
    pub code: String,
    pub name: String,
    pub input_methods: Vec<KeyboardLayout>,
}
