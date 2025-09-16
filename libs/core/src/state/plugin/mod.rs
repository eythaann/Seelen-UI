pub mod value;

use std::{fs::File, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use value::PluginValue;

use crate::{
    error::Result,
    resource::{ConcreteResource, PluginId, ResourceMetadata, SluResource, SluResourceFile},
    utils::search_for_metadata_file,
};

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
    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ResourceMetadata {
        &mut self.metadata
    }

    fn load_from_file(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .ok_or("Invalid file extension")?
            .to_string_lossy();

        let plugin = match ext.as_ref() {
            "yml" | "yaml" => serde_yaml::from_reader(File::open(path)?)?,
            "json" | "jsonc" => serde_json::from_reader(File::open(path)?)?,
            "slu" => {
                let file = SluResourceFile::load(path)?;
                match file.concrete()? {
                    ConcreteResource::Plugin(plugin) => plugin,
                    _ => return Err("Resource file is not a plugin".into()),
                }
            }
            _ => return Err("Invalid file extension".into()),
        };

        Ok(plugin)
    }

    fn load_from_folder(path: &Path) -> Result<Self> {
        let file = search_for_metadata_file(path).ok_or("No metadata file found")?;
        Self::load_from_file(&file)
    }
}

impl Plugin {
    pub fn default_icon() -> String {
        "PiPuzzlePieceDuotone".to_string()
    }
}
