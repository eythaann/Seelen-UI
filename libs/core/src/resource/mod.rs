mod file;
mod interface;
mod resource_id;
mod yaml_ext;

pub use file::*;
pub use interface::*;
pub use resource_id::*;
pub use yaml_ext::*;

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::PathBuf,
};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use url::Url;
use uuid::Uuid;

use crate::error::Result;

// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
/// Map of language code as key an translated values. Could be a string, mapped to `en`.
pub enum ResourceText {
    En(String),
    Localized(HashMap<String, String>),
}

impl ResourceText {
    const MISSING_TEXT: &'static str = "!?";

    /// Returns true if the text exists for the given lang
    pub fn has(&self, lang: &str) -> bool {
        match self {
            ResourceText::En(_) => lang == "en",
            ResourceText::Localized(map) => map.get(lang).is_some_and(|t| !t.is_empty()),
        }
    }

    /// Returns the text by lang, uses `en` as fallback.
    /// If no text fallback found will return `!?`
    pub fn get(&self, lang: &str) -> &str {
        match self {
            ResourceText::En(value) => value,
            ResourceText::Localized(map) => match map.get(lang) {
                Some(value) => value,
                None => match map.get("en") {
                    Some(value) => value,
                    None => Self::MISSING_TEXT,
                },
            },
        }
    }

    pub fn set(&mut self, lang: impl Into<String>, value: impl Into<String>) {
        if let ResourceText::En(v) = self {
            let mut dict = HashMap::new();
            dict.insert("en".to_string(), v.to_string());
            *self = ResourceText::Localized(dict);
        }

        if let ResourceText::Localized(dict) = self {
            dict.insert(lang.into(), value.into());
        }
    }
}

// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
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

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct InternalResourceMetadata {
    pub path: PathBuf,
    pub filename: String,
    pub bundled: bool,
    /// Last date when the metadata file was written
    pub written_at: DateTime<Utc>,
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

// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum ResourceKind {
    Theme,
    IconPack,
    Widget,
    Plugin,
    Wallpaper,
    SoundPack,
}

// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum ResourceStatus {
    /// Initial state
    Draft,
    /// Waiting for review
    Reviewing,
    /// review done and rejected
    Rejected,
    /// review done and approved
    Published,
    /// soft delete by user
    Deleted,
}

// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum ResourceAttribute {
    /// this resource is not working
    NotWorking,
    /// this resource is recommended by the staff
    StaffLiked,
}

// =============================================================================

/// Represents a resource in the cloud, uploaded by a user
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Resource {
    /// unique id
    pub id: Uuid,
    /// id of the document containing the resource
    pub data_id: Uuid,
    /// user id who created the resource
    pub creator_id: Uuid,
    /// visual id composed of creator username and resource name
    pub friendly_id: ResourceId,
    pub kind: ResourceKind,
    pub metadata: ResourceMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    /// current status
    pub status: ResourceStatus,
    /// if status == ResourceStatus::Rejected, this is the reason for rejection
    pub rejected_reason: Option<String>,
    /// date when the resource was reviewed
    pub reviewed_at: Option<DateTime<Utc>>,
    /// user id who reviewed the resource
    pub reviewed_by: Option<Uuid>,
    /// should be filled if `status == ResourceStatus::Deleted`
    pub deleted_at: Option<DateTime<Utc>>,

    /// resource attributes
    #[serde(default)]
    pub attributes: HashSet<ResourceAttribute>,
    /// resource version (increased every time the resource is updated)
    pub version: u32,
    /// number of stars
    pub stars: u32,
    /// number of downloads
    pub downloads: u32,
}

impl Resource {
    pub fn verify(&self) -> Result<()> {
        if let ResourceText::Localized(map) = &self.metadata.display_name {
            if map.get("en").is_none() {
                return Err("missing mandatory english display name".into());
            }
        }

        if let ResourceText::Localized(map) = &self.metadata.description {
            if map.get("en").is_none() {
                return Err("missing mandatory english description".into());
            }
        }
        Ok(())
    }
}
