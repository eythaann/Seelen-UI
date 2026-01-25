use std::{collections::HashMap, path::PathBuf};

use chrono::{DateTime, Utc};
use url::Url;

use crate::{
    error::Result,
    resource::{Resource, ResourceText},
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct ResourceMetadata {
    /// Map of language code to display name. Could be a string, mapped to `en`.
    pub display_name: ResourceText,
    /// Map of language code to description. Could be a string, mapped to `en`.
    pub description: ResourceText,
    /// Portrait image with aspect ratio of 1/1
    pub portrait: Option<Url>,
    /// Banner image with aspect ratio of 21/9, this is used when promoting the resource.
    pub banner: Option<Url>,
    /// Screenshots should use aspect ratio of 16/9
    pub screenshots: Vec<Url>,
    /// tags are keywords to be used for searching and indexing
    pub tags: Vec<String>,
    /// App target version that this resource is compatible with.\
    /// Developers are responsible to update the resource so when resource does not
    /// match the current app version, the resource will be shown with a warning message
    pub app_target_version: Option<(u32, u32, u32)>,
    /// Extra metadata for the resource
    pub extras: HashMap<String, String>,
    #[serde(flatten, skip_deserializing)]
    pub internal: InternalResourceMetadata,
}

impl ResourceMetadata {
    /// Returns the directory of where the resource is stored
    pub fn directory(&self) -> Result<PathBuf> {
        Ok(if self.internal.path.is_dir() {
            self.internal.path.clone()
        } else {
            self.internal
                .path
                .parent()
                .ok_or("No Parent")?
                .to_path_buf()
        })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct InternalResourceMetadata {
    pub path: PathBuf,
    pub filename: String,
    pub bundled: bool,
    /// Last date when the metadata file was written
    pub written_at: DateTime<Utc>,
    /// only present for remote/downloaded resources
    pub remote: Option<Box<Resource>>,
}

impl Default for ResourceMetadata {
    fn default() -> Self {
        Self {
            display_name: ResourceText::Localized(HashMap::new()),
            description: ResourceText::Localized(HashMap::new()),
            portrait: None,
            banner: None,
            screenshots: Vec::new(),
            tags: Vec::new(),
            extras: HashMap::new(),
            app_target_version: None,
            internal: InternalResourceMetadata::default(),
        }
    }
}
