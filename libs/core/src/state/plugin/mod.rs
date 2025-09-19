pub mod value;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use value::PluginValue;

use crate::resource::{PluginId, ResourceKind, ResourceMetadata, SluResource};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Plugin {
    pub id: PluginId,
    #[serde(default)]
    pub metadata: ResourceMetadata,
    /// Optional icon to be used on settings. This have to be a valid react icon name.\
    /// You can find all icons here: https://react-icons.github.io/react-icons/.
    #[serde(default = "Plugin::default_icon")]
    pub icon: String,
    #[serde(flatten)]
    pub plugin: PluginValue,
}

impl SluResource for Plugin {
    const KIND: ResourceKind = ResourceKind::Plugin;

    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ResourceMetadata {
        &mut self.metadata
    }
}

impl Plugin {
    pub fn default_icon() -> String {
        "PiPuzzlePieceDuotone".to_string()
    }
}
