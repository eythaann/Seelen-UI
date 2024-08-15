use serde::{Deserialize, Serialize};

use crate::state::domain::{Placeholder, Theme};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub resources: ResourceItems,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceItems {
    pub theme: Option<Theme>,
    pub placeholder: Option<Placeholder>,
    pub layout: Option<serde_yaml::Value>,
}
