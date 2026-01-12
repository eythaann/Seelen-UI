use std::{fs::File, path::Path};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::Result,
    resource::{deserialize_extended_yaml, ResourceKind, SluResourceFile},
    utils::search_resource_entrypoint,
};

use super::ResourceMetadata;

pub trait SluResource: Sized + Serialize + DeserializeOwned {
    const KIND: ResourceKind;

    fn metadata(&self) -> &ResourceMetadata;
    fn metadata_mut(&mut self) -> &mut ResourceMetadata;

    /// Try to load the resource from a file.\
    /// This won't run post loading processing, please use `load` instead.
    fn load_from_file(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .ok_or("Invalid file extension")?
            .to_ascii_lowercase();

        let resource: Self = match ext.to_string_lossy().as_ref() {
            "yml" | "yaml" => deserialize_extended_yaml(path)?,
            "json" | "jsonc" => {
                let file = File::open(path)?;
                file.lock_shared()?;
                serde_json::from_reader(file)?
            }
            "slu" => {
                let file = SluResourceFile::load(path)?;
                if Self::KIND != file.resource.kind {
                    return Err(format!(
                        "Resource file is not of expected kind: {:?} instead is {:?}",
                        Self::KIND,
                        file.resource.kind
                    )
                    .into());
                }
                file.try_parse_into()?
            }
            _ => return Err("Invalid file extension".into()),
        };

        Ok(resource)
    }

    /// Try to load the resource from a folder.\
    /// This won't run post loading processing, please use `load` instead.
    fn load_from_folder(path: &Path) -> Result<Self> {
        let file = search_resource_entrypoint(path).ok_or("No metadata file found")?;
        Self::load_from_file(&file)
    }

    /// Try to load the resource from a file or directory.\
    /// After deserialization, this will run post loading processing like `sanitize` and `validate`,
    /// Also will set the internal metadata needed to handle the resource
    fn load(path: &Path) -> Result<Self> {
        let mut resource = if path.is_dir() {
            Self::load_from_folder(path)?
        } else {
            Self::load_from_file(path)?
        };

        let meta = resource.metadata_mut();
        meta.internal.path = path.to_path_buf();
        meta.internal.filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        meta.internal.written_at = path.metadata()?.modified()?.into();

        resource.sanitize();
        resource.validate()?;
        Ok(resource)
    }

    /// Sanitize the resource data
    fn sanitize(&mut self) {}

    /// Validates the resource after sanitization
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Saves the resource in same path as it was loaded
    fn save(&self) -> Result<()> {
        let mut save_path = self.metadata().internal.path.to_path_buf();
        if save_path.is_dir() {
            std::fs::create_dir_all(&save_path)?;
            save_path = search_resource_entrypoint(&save_path)
                .unwrap_or_else(|| save_path.join("metadata.yml"));
        }

        let extension = save_path
            .extension()
            .ok_or("Invalid path extension")?
            .to_string_lossy()
            .to_lowercase();

        match extension.as_str() {
            "slu" => {
                let mut slu_file = SluResourceFile::load(&save_path)?;
                slu_file.data = serde_json::to_value(self)?.into();
                slu_file.store(&save_path)?;
            }
            "yml" | "yaml" => {
                let file = File::create(save_path)?;
                serde_yaml::to_writer(file, self)?;
            }
            "json" | "jsonc" => {
                let file = File::create(save_path)?;
                serde_json::to_writer_pretty(file, self)?;
            }
            _ => {
                return Err("Unsupported path extension".into());
            }
        }
        Ok(())
    }

    fn delete(&self) -> Result<()> {
        let path = self.metadata().internal.path.to_path_buf();
        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}
