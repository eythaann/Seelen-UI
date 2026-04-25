use crate::{resource::ResourceText, state::Alignment, utils::TsUnknown};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export, optional_fields = nullable))]
pub struct ContextMenu {
    pub identifier: uuid::Uuid,
    /// Optional metadata that will be sent back in the callback payload when an item is clicked.
    pub meta: Option<TsUnknown>,
    /// Items of the context menu
    pub items: Vec<ContextMenuItem>,
    /// Alignment of the context menu on the X axis relative to the trigger point.
    pub align_x: Option<Alignment>,
    /// Alignment of the context menu on the Y axis relative to the trigger point.
    pub align_y: Option<Alignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all_fields = "camelCase")]
#[ts(optional_fields = nullable)]
pub enum ContextMenuItem {
    Separator,
    Item {
        key: String,
        value: Option<TsUnknown>,
        icon: Option<String>,
        label: String,
        /// event name to be emitted on click, `key` will be sent as payload
        callback_event: String,
        /// If not null, the item will display a checkbox.
        /// `checked` field will be send as payload.
        checked: Option<bool>,
        disabled: Option<bool>,
    },
    Submenu {
        identifier: uuid::Uuid,
        icon: Option<String>,
        label: ResourceText,
        items: Vec<ContextMenuItem>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export, optional_fields = nullable))]
pub struct ContextMenuCallbackPayload {
    /// The key of the clicked item
    key: String,
    /// The value of the clicked item, if any
    value: Option<TsUnknown>,
    /// The checked state of the clicked item, if it has a checkbox
    checked: Option<bool>,
    /// The meta of the context menu
    meta: Option<TsUnknown>,
}
