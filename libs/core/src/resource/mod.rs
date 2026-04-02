mod file;
mod interface;
mod metadata;
mod resource_id;
mod yaml_ext;

pub use file::*;
pub use interface::*;
pub use metadata::*;
pub use resource_id::*;
pub use yaml_ext::*;

pub use crate::utils::Slug;

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
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

impl Default for ResourceText {
    fn default() -> Self {
        Self::En(String::new())
    }
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

/// Represents a pending update submitted by the creator of a Published resource.
/// The live resource remains unchanged until the update is approved or rejected.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct ResourcePendingUpdate {
    /// Proposed new metadata (applied to the resource on approve)
    pub metadata: ResourceMetadata,
    /// ID of the pending resource data document in the type-specific collection
    pub data_id: Uuid,
    /// When the pending update was submitted
    pub updated_at: DateTime<Utc>,
}

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

    /// slug to shown in the url, could be changed.
    #[serde(default)]
    pub slug: Slug,
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
    /// Pending update waiting for reviewer approval (only present on Published resources)
    pub pending_update: Option<ResourcePendingUpdate>,
    /// Reason a pending update was rejected (visible to creator after rejection)
    pub rejected_pending_reason: Option<String>,

    /// @deprecated - used for migrations only
    #[ts(skip)]
    #[serde(rename = "friendlyId")]
    pub deprecated_id: Option<ResourceId>,
}

impl Resource {
    /// Fills in missing fields from available data. Call before `verify`.
    /// Currently: derives `slug` from `friendly_id` when `slug` is empty.
    pub fn sanitize(&mut self) {
        if self.slug.is_empty() {
            if let Some(id) = &self.deprecated_id {
                if let Some(name) = id.resource_name() {
                    self.slug = Slug::from_lossy(&name);
                }
            }
        }
    }

    pub fn verify(&self) -> Result<()> {
        if self.slug.is_empty() {
            return Err("missing slug".into());
        }

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
