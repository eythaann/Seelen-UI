use std::{collections::HashSet, path::PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::system_state::{Relaunch, RelaunchArguments};

#[derive(Debug, Default, Clone, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WegItemData {
    /// internal UUID to differentiate items
    pub id: uuid::Uuid,
    /// display name of the item
    pub display_name: String,
    /// Application user model id.
    pub umid: Option<String>,
    /// path to file or program.
    pub path: PathBuf,
    /// the item will persist after all windows are closed
    pub pinned: bool,
    /// this item should not be pinnable
    pub prevent_pinning: bool,
    /// custom information to relaunch this app, if none, UMID or path should be used
    pub relaunch: Option<Relaunch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
pub enum WegItem {
    #[serde(alias = "PinnedApp", alias = "Pinned")]
    DeprecatedOldPinned(OldWegItemData),
    AppOrFile(WegItemData),
    Separator {
        id: uuid::Uuid,
    },
    Media {
        id: uuid::Uuid,
    },
    StartMenu {
        id: uuid::Uuid,
    },
    ShowDesktop {
        id: uuid::Uuid,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export, repr(enum = name)))]
pub enum WegItemType {
    AppOrFile,
    Separator,
    Media,
    StartMenu,
    ShowDesktop,
}

impl WegItem {
    pub fn id(&self) -> &uuid::Uuid {
        match self {
            WegItem::DeprecatedOldPinned(data) => &data.id,
            WegItem::AppOrFile(data) => &data.id,
            WegItem::Separator { id } => id,
            WegItem::Media { id } => id,
            WegItem::StartMenu { id } => id,
            WegItem::ShowDesktop { id } => id,
        }
    }

    fn set_id(&mut self, identifier: uuid::Uuid) {
        match self {
            WegItem::DeprecatedOldPinned(data) => data.id = identifier,
            WegItem::AppOrFile(data) => data.id = identifier,
            WegItem::Separator { id } => *id = identifier,
            WegItem::Media { id } => *id = identifier,
            WegItem::StartMenu { id } => *id = identifier,
            WegItem::ShowDesktop { id } => *id = identifier,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct WegItems {
    /// Whether the reordering possible on the weg
    pub is_reorder_disabled: bool,
    pub left: Vec<WegItem>,
    pub center: Vec<WegItem>,
    pub right: Vec<WegItem>,
}

impl WegItems {
    fn migrate_item(item: WegItem) -> Option<WegItem> {
        let WegItem::DeprecatedOldPinned(mut data) = item else {
            return Some(item);
        };

        // migration step for items before v2.1.6
        if data.subtype == OldWegItemSubtype::UnknownV2_1_6 {
            data.subtype = if data.relaunch_program.to_lowercase().contains(".exe") {
                OldWegItemSubtype::App
            } else if data.path.is_dir() {
                OldWegItemSubtype::Folder
            } else {
                OldWegItemSubtype::File
            };
        }

        if data.subtype == OldWegItemSubtype::Folder {
            return None;
        }

        // migration of old scheme before v2.5
        if let Some(args) = &data.relaunch_args {
            if data.relaunch_program.contains("explorer")
                && args.to_string().starts_with("shell:AppsFolder")
            {
                data.relaunch_program = args.to_string();
                data.relaunch_args = None;
            }
        }

        if data.relaunch_program.is_empty() {
            data.relaunch_program = data.path.to_string_lossy().to_string();
        }

        let relaunch = Some(Relaunch {
            command: data.relaunch_program,
            args: data.relaunch_args,
            working_dir: data.relaunch_in,
            icon: None,
        });

        Some(WegItem::AppOrFile(WegItemData {
            id: data.id,
            display_name: data.display_name,
            umid: data.umid,
            path: data.path,
            pinned: true,
            prevent_pinning: data.pin_disabled,
            relaunch,
        }))
    }

    fn migrate_items(items: Vec<WegItem>) -> Vec<WegItem> {
        items.into_iter().filter_map(Self::migrate_item).collect()
    }

    pub fn migrate(&mut self) {
        self.left = Self::migrate_items(std::mem::take(&mut self.left));
        self.center = Self::migrate_items(std::mem::take(&mut self.center));
        self.right = Self::migrate_items(std::mem::take(&mut self.right));
    }

    fn sanitize_items(dict: &mut HashSet<uuid::Uuid>, items: Vec<WegItem>) -> Vec<WegItem> {
        let mut result = Vec::new();
        for mut item in items {
            if let WegItem::AppOrFile(data) = &item {
                let should_ensure_path =
                    data.umid.is_none() || data.path.extension().is_some_and(|e| e == "lnk");
                if should_ensure_path && !data.path.exists() {
                    continue;
                }
            }

            if item.id().is_nil() {
                item.set_id(uuid::Uuid::new_v4());
            }

            if !dict.contains(item.id()) {
                dict.insert(*item.id());
                result.push(item);
            }
        }
        result
    }

    pub fn sanitize(&mut self) {
        self.migrate();
        let mut dict = HashSet::new();
        self.left = Self::sanitize_items(&mut dict, std::mem::take(&mut self.left));
        self.center = Self::sanitize_items(&mut dict, std::mem::take(&mut self.center));
        self.right = Self::sanitize_items(&mut dict, std::mem::take(&mut self.right));
    }
}

// ===================== DEPRECATED STRUCTS =====================

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(repr(enum = name))]
pub enum OldWegItemSubtype {
    File,
    Folder,
    App,
    #[default]
    UnknownV2_1_6,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct OldWegItemData {
    /// internal UUID to differentiate items
    pub id: uuid::Uuid,
    /// Subtype of the item (mandatory, but is optional for backward compatibility)
    pub subtype: OldWegItemSubtype,
    /// Application user model id.
    pub umid: Option<String>,
    /// path to file, folder or program.
    pub path: PathBuf,
    /// program to be executed
    pub relaunch_program: String,
    /// arguments to be passed to the relaunch program
    pub relaunch_args: Option<RelaunchArguments>,
    /// path where ejecute the relaunch command
    pub relaunch_in: Option<PathBuf>,
    /// display name of the item
    pub display_name: String,
    /// This intention is to prevent pinned state change, when this is neccesary
    #[serde(skip_deserializing)]
    pub pin_disabled: bool,
}
