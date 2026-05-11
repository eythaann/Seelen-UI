use std::{fs::File, path::Path};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::Result,
    resource::{
        deserialize_extended_yaml, deserialize_extended_yaml_no_vars, ResourceKind, SluResourceFile,
    },
    utils::search_resource_entrypoint,
};

use super::ResourceMetadata;

pub trait SluResource: Sized + Serialize + DeserializeOwned {
    const KIND: ResourceKind;

    fn metadata(&self) -> &ResourceMetadata;
    fn metadata_mut(&mut self) -> &mut ResourceMetadata;

    fn load_from_file(path: &Path, resolve_self_vars: bool) -> Result<Self> {
        let ext = path
            .extension()
            .ok_or("Invalid file extension")?
            .to_ascii_lowercase();

        let resource: Self = match ext.to_string_lossy().as_ref() {
            "yml" | "yaml" => {
                if resolve_self_vars {
                    deserialize_extended_yaml(path)?
                } else {
                    deserialize_extended_yaml_no_vars(path)?
                }
            }
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

                let mut parsed: Self = file.try_parse_into()?;
                parsed.metadata_mut().internal.remote = Some(Box::new(file.resource.clone()));
                parsed
            }
            _ => return Err("Invalid file extension".into()),
        };

        Ok(resource)
    }

    fn load_from_folder(path: &Path, resolve_self_vars: bool) -> Result<Self> {
        let file = search_resource_entrypoint(path).ok_or("No metadata file found")?;
        Self::load_from_file(&file, resolve_self_vars)
    }

    fn load_ext(path: &Path, resolve_self_vars: bool) -> Result<Self> {
        let mut resource = if path.is_dir() {
            Self::load_from_folder(path, resolve_self_vars)?
        } else {
            Self::load_from_file(path, resolve_self_vars)?
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

    fn load(path: &Path) -> Result<Self> {
        Self::load_ext(path, true)
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
