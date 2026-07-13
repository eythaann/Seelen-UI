pub mod twm;
pub mod value;
pub mod weg;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use value::PluginValue;

use crate::resource::{PluginId, ResourceKind, ResourceMetadata, SluResource};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct Plugin {
    pub id: PluginId,
    pub metadata: ResourceMetadata,
    /// Optional icon to be used on settings. This have to be a valid react icon name.\
    /// You can find all icons here: https://react-icons.github.io/react-icons/.
    pub icon: String,
    #[serde(flatten)]
    pub plugin: PluginValue,
}

impl Default for Plugin {
    fn default() -> Self {
        Self {
            id: Default::default(),
            metadata: Default::default(),
            icon: "PiPuzzlePieceDuotone".to_string(),
            plugin: PluginValue::Any(Default::default()),
        }
    }
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
