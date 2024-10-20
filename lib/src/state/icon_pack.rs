use std::{collections::HashMap, path::PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::ResourceMetadata;

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct IconPack {
    pub info: ResourceMetadata,
    /// Key can be user model id, filename or a full path.
    /// 
    /// Value is the path to the icon relative to the icon pack folder.
    pub apps: HashMap<String, PathBuf>,
}
