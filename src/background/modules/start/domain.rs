use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartMenuItem {
    pub path: PathBuf,
    pub umid: Option<String>,
    pub target: Option<PathBuf>,
}
