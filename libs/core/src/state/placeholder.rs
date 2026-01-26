use std::collections::{HashMap, HashSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use url::Url;

use crate::{resource::PluginId, utils::TsUnknown};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum ToolbarJsScope {
    Date,
    Notifications,
    Media,
    Network,
    Keyboard,
    User,
    Bluetooth,
    Power,
    FocusedApp,
    Workspaces,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct ToolbarItem {
    /// Id to identify the item, should be unique. Preferably a uuid.
    pub id: String,
    /// List of scopes to be loaded in the item js execution scope.
    pub scopes: HashSet<ToolbarJsScope>,
    /// JS function definition for content to display in the item.
    pub template: String,
    /// JS function definition for content to display in tooltip of the item.
    pub tooltip: Option<String>,
    /// JS function definition badge content, will be displayed over the item, useful as notifications.
    pub badge: Option<String>,
    /// JS function definition that will be executed when the item is clicked.
    #[serde(alias = "onClickV2")]
    pub on_click: Option<String>,
    /// Styles to be added to the item. This follow the same interface of React's `style` prop.
    pub style: HashMap<String, Option<StyleValue>>,
    /// Remote data to be added to the item scope.
    pub remote_data: HashMap<String, RemoteDataDeclaration>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoteDataDeclaration {
    url: Url,
    request_init: Option<TsUnknown>,
    update_interval_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
pub enum StyleValue {
    String(String),
    Number(serde_json::Number),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(repr(enum = name))]
pub enum WorkspaceToolbarItemMode {
    #[default]
    Dotted,
    Named,
    Numbered,
}

impl ToolbarItem {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
pub enum ToolbarItem2 {
    Plugin(PluginId),
    Inline(Box<ToolbarItem>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct ToolbarState {
    /// Whether the reordering possible on the toolbar
    pub is_reorder_disabled: bool,
    /// Items to be displayed in the toolbar
    pub left: Vec<ToolbarItem2>,
    /// Items to be displayed in the toolbar
    pub center: Vec<ToolbarItem2>,
    /// Items to be displayed in the toolbar
    pub right: Vec<ToolbarItem2>,
}

impl ToolbarState {
    fn migrate_plugin_id(id: PluginId) -> PluginId {
        match id.as_str() {
            "@default/system-tray" => "@seelen/tb-system-tray".into(),
            "@default/quick-settings" => "@seelen/tb-quick-settings".into(),
            "@default/bluetooth" => "@seelen/tb-bluetooth-popup".into(),
            "@default/keyboard" => "@seelen/tb-keyboard-selector".into(),
            "@default/user" => "@seelen/tb-user-menu".into(),
            "@default/network" => "@seelen/tb-network-popup".into(),
            "@default/date" => "@seelen/tb-calendar-popup".into(),
            "@default/media" => "@seelen/tb-media-popup".into(),
            "@default/notifications" => "@seelen/tb-notifications".into(),
            _ => id,
        }
    }

    fn sanitize_items(dict: &mut HashSet<String>, items: Vec<ToolbarItem2>) -> Vec<ToolbarItem2> {
        let mut result = Vec::new();
        for item in items {
            match item {
                ToolbarItem2::Plugin(id) => {
                    let id = Self::migrate_plugin_id(id);
                    let str_id = id.to_string();

                    if !dict.contains(&str_id) && id.is_valid() {
                        dict.insert(str_id);
                        result.push(ToolbarItem2::Plugin(id));
                    }
                }
                ToolbarItem2::Inline(mut item) => {
                    // migration step for old default separator before v2.5
                    if item.template.contains("window") && item.scopes.is_empty() {
                        item.scopes.insert(ToolbarJsScope::FocusedApp);
                        item.template = item.template.replace("window", "focusedApp");
                    }

                    if item.id().is_empty() {
                        item.set_id(uuid::Uuid::new_v4().to_string());
                    }

                    if !dict.contains(&item.id()) {
                        dict.insert(item.id());
                        result.push(ToolbarItem2::Inline(item));
                    }
                }
            }
        }
        result
    }

    pub fn sanitize(&mut self) {
        let mut dict = HashSet::new();
        self.left = Self::sanitize_items(&mut dict, std::mem::take(&mut self.left));
        self.center = Self::sanitize_items(&mut dict, std::mem::take(&mut self.center));
        self.right = Self::sanitize_items(&mut dict, std::mem::take(&mut self.right));
    }
}
