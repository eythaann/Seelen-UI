use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StartMenuItem {
    pub path: PathBuf,
    pub umid: Option<String>,
}
