use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use base64::Engine;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    error::Result,
    state::{IconPack, Plugin, Theme, Wallpaper, Widget},
    utils::TsUnknown,
};

use super::{Resource, ResourceKind};

/// A container for Seelen UI resources.
///
/// This struct contains all the necessary data that a resource needs.
/// It uses a custom `.slu` file extension format that can change over time
/// with new versions.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct SluResourceFile {
    pub version: u32,
    pub resource: Resource,
    pub data: TsUnknown,
}

pub enum ConcreteResource {
    Theme(Theme),
    Plugin(Plugin),
    IconPack(IconPack),
    Widget(Widget),
    Wallpaper(Wallpaper),
}

impl SluResourceFile {
    pub fn decode<R: Read + Seek>(mut reader: R) -> Result<Self> {
        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;

        match version[0] {
            1 => {
                reader.seek(SeekFrom::Current(3))?; // SLU mime type
            }
            2 => {
                reader.seek(SeekFrom::Current(3))?; // SLU mime type
                reader.seek(SeekFrom::Current(4))?; // 32 bits reserved
            }
            // todo for version 3, use zip crate
            _ => {
                return Err("unsupported slu file version".into());
            }
        }

        // read the rest of the body as content
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        let decoded = base64::engine::general_purpose::STANDARD.decode(&buffer)?;
        Ok(serde_yaml::from_slice(&decoded)?)
    }

    pub fn encode<W: Write>(&self, mut writer: W) -> Result<()> {
        let data = serde_yaml::to_string(self)?;
        let encoded = base64::engine::general_purpose::STANDARD.encode(data);

        writer.write_all(&[2])?; // version
        writer.write_all("SLU".as_bytes())?; // SLU mime type
        writer.write_all(&[0u8; 4])?; // 32 bits reserved
        writer.write_all(encoded.as_bytes())?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let decoded = Self::decode(&file)?;
        decoded.resource.verify()?;
        Ok(decoded)
    }

    pub fn store(&self, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;
        self.encode(&mut file)
    }

    pub fn concrete(&self) -> Result<ConcreteResource> {
        let mut resource = serde_json::value::Map::new();

        resource.insert(
            "id".to_string(),
            serde_json::Value::String(self.resource.friendly_id.to_string()),
        );
        resource.insert(
            "metadata".to_string(),
            serde_json::to_value(&self.resource.metadata)?,
        );

        let data = self.data.0.as_object().ok_or("invalid data")?;
        resource.append(&mut data.clone());

        let concrete = match self.resource.kind {
            ResourceKind::Theme => {
                ConcreteResource::Theme(serde_json::from_value(resource.into())?)
            }
            ResourceKind::Plugin => {
                ConcreteResource::Plugin(serde_json::from_value(resource.into())?)
            }
            ResourceKind::IconPack => {
                ConcreteResource::IconPack(serde_json::from_value(resource.into())?)
            }
            ResourceKind::Widget => {
                ConcreteResource::Widget(serde_json::from_value(resource.into())?)
            }
            _ => return Err("unsupported resource kind".into()),
        };

        Ok(concrete)
    }
}
