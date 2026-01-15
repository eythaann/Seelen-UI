use std::path::PathBuf;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::resource::{IconPackId, ResourceKind, ResourceMetadata, SluResource};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct IconPack {
    pub id: IconPackId,
    #[serde(alias = "info")]
    pub metadata: ResourceMetadata,
    /// Special icon used when some other icon is not found
    pub missing: Option<Icon>,
    /// Icons defined in this icon pack
    pub entries: Vec<IconPackEntry>,
    /// This lists will be downloaded and stored locally
    pub remote_entries: Vec<IconPackEntry>,
    /// Indicates if the icon pack icons was downloaded from `remote_entries`
    pub downloaded: bool,
}

impl SluResource for IconPack {
    const KIND: ResourceKind = ResourceKind::IconPack;

    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ResourceMetadata {
        &mut self.metadata
    }

    fn sanitize(&mut self) {
        self.missing = self.missing.take().filter(|e| e.is_valid());
        self.entries.retain(|e| match e {
            IconPackEntry::Unique(e) => match &e.icon {
                Some(icon) => icon.is_valid(),
                None => e.redirect.is_some(),
            },
            IconPackEntry::Shared(e) => e.icon.is_valid(),
            IconPackEntry::Custom(e) => e.icon.is_valid(),
        })
    }
}

impl IconPack {
    /// replace existing entry if found, otherwise add it.
    pub fn add_entry(&mut self, entry: IconPackEntry) {
        if let Some(found) = self.find_similar_mut(&entry) {
            *found = entry;
        } else {
            self.entries.push(entry);
        }
    }

    /// search for same entry ignoring the icon.
    pub fn find_similar_mut(&mut self, entry: &IconPackEntry) -> Option<&mut IconPackEntry> {
        self.entries
            .iter_mut()
            .find(|existing| existing.matches(entry))
    }

    /// search for same entry ignoring the icon.
    pub fn find_similar(&self, entry: &IconPackEntry) -> Option<&IconPackEntry> {
        self.entries.iter().find(|existing| existing.matches(entry))
    }

    /// search for same entry ignoring the icon.
    pub fn contains_similar(&self, entry: &IconPackEntry) -> bool {
        self.find_similar(entry).is_some()
    }
}

/// Key can be user model id, filename or a full path.
/// In case of path this should be an executable or a lnk file or any other file that can
/// have an unique/individual icon as are the applications, otherwise use `shared`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct UniqueIconPackEntry {
    /// Application user model id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub umid: Option<String>,
    /// Path or filename of the application, mostly this should be present,
    /// but cases like PWAs on Edge can have no path and be only an UMID.
    pub path: Option<PathBuf>,
    /// In case of path be a lnk file this can be set to a different location to use the icon from.
    /// If present, icon on this entry will be ignored
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,
    /// Source file modification time for cache invalidation
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional = nullable)]
    pub source_mtime: Option<DateTime<Utc>>,
}

/// Intended to store file icons by extension
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct SharedIconPackEntry {
    /// File extension without the dot, e.g. "txt"
    pub extension: String,
    pub icon: Icon,
}

/// Here specific/custom icons for widgets can be stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct CustomIconPackEntry {
    /// we recomend following the widget id + icon name to avoid collisions
    /// e.g. "@username/widgetid::iconname" but you can use whatever you want
    pub key: String,
    /// Value is the path to the icon relative to the icon pack folder.
    pub icon: Icon,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum IconPackEntry {
    Unique(UniqueIconPackEntry),
    Shared(SharedIconPackEntry),
    Custom(CustomIconPackEntry),
}

impl IconPackEntry {
    pub fn matches(&self, entry: &IconPackEntry) -> bool {
        match (self, entry) {
            (IconPackEntry::Unique(self_unique), IconPackEntry::Unique(other_unique)) => {
                self_unique.umid == other_unique.umid
                    && self_unique.path == other_unique.path
                    && self_unique.redirect == other_unique.redirect
            }
            (IconPackEntry::Shared(self_shared), IconPackEntry::Shared(other_shared)) => {
                self_shared.extension == other_shared.extension
            }
            (IconPackEntry::Custom(self_custom), IconPackEntry::Custom(other_custom)) => {
                self_custom.key == other_custom.key
            }
            _ => false,
        }
    }
}

/// The icon paths in this structure are relative to the icon pack folder.
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct Icon {
    /// Icon to use if no light or dark icon is specified, if both light and dark are specified this can be omitted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    /// Alternative icon to use when system theme is light
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light: Option<String>,
    /// Alternative icon to use when system theme is dark
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dark: Option<String>,
    /// Mask to be applied over the icon, themes can use this to apply custom colors over the icon.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask: Option<String>,
    /// Whether the icon is a square or not
    #[serde(skip_serializing_if = "is_false")]
    pub is_aproximately_square: bool,
}

impl Icon {
    pub fn is_valid(&self) -> bool {
        self.base.is_some() || (self.light.is_some() && self.dark.is_some())
    }
}

fn is_false(b: &bool) -> bool {
    !b
}
