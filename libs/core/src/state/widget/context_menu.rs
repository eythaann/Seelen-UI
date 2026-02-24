use crate::{resource::ResourceText, state::Alignment};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export, optional_fields = nullable))]
pub struct ContextMenu {
    pub identifier: uuid::Uuid,
    pub items: Vec<ContextMenuItem>,
    /// Alignment of the context menu on the X axis relative to the trigger point.
    pub align_x: Option<Alignment>,
    /// Alignment of the context menu on the Y axis relative to the trigger point.
    pub align_y: Option<Alignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all_fields = "camelCase")]
pub enum ContextMenuItem {
    Separator,
    Item {
        key: String,
        icon: Option<String>,
        label: String,
        /// event name to be emitted on click, `key` will be sent as payload
        callback_event: String,
        /// If not null, the item will display a checkbox.
        /// `checked` field will be send as payload.
        #[ts(optional = nullable)]
        checked: Option<bool>,
        #[ts(optional = nullable)]
        disabled: Option<bool>,
    },
    Submenu {
        identifier: uuid::Uuid,
        icon: Option<String>,
        label: ResourceText,
        items: Vec<ContextMenuItem>,
    },
}
