use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    error::Result,
    resource::{
        deserialize_extended_yaml, deserialize_extended_yaml_no_vars, ResourceKind, SluResourceFile,
    },
    utils::search_resource_entrypoint,
};

use super::ResourceMetadata;

#[allow(async_fn_in_trait)]
pub trait SluResource: Sized + Serialize + DeserializeOwned {
    const KIND: ResourceKind;

    fn metadata(&self) -> &ResourceMetadata;
    fn metadata_mut(&mut self) -> &mut ResourceMetadata;

    async fn load_from_file(path: &Path, resolve_self_vars: bool) -> Result<Self> {
        let ext = path
            .extension()
            .ok_or("Invalid file extension")?
            .to_ascii_lowercase();

        let resource: Self = match ext.to_string_lossy().as_ref() {
            "yml" | "yaml" => {
                if resolve_self_vars {
                    deserialize_extended_yaml(path).await?
                } else {
                    deserialize_extended_yaml_no_vars(path).await?
                }
            }
            "json" | "jsonc" => {
                let bytes = tokio::fs::read(path).await?;
                serde_json::from_slice(&bytes)?
            }
            "slu" => {
                let file = SluResourceFile::load(path).await?;
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

    async fn load_from_folder(path: &Path, resolve_self_vars: bool) -> Result<Self> {
        let file = search_resource_entrypoint(path)
            .await
            .ok_or("No metadata file found")?;
        Self::load_from_file(&file, resolve_self_vars).await
    }

    async fn load_ext(path: &Path, resolve_self_vars: bool) -> Result<Self> {
        let fs_meta = tokio::fs::metadata(path).await?;
        let mut resource = if fs_meta.is_dir() {
            Self::load_from_folder(path, resolve_self_vars).await?
        } else {
            Self::load_from_file(path, resolve_self_vars).await?
        };

        let meta = resource.metadata_mut();
        meta.internal.path = path.to_path_buf();
        meta.internal.filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        meta.internal.written_at = fs_meta.modified()?.into();

        resource.sanitize();
        resource.validate()?;
        Ok(resource)
    }

    async fn load(path: &Path) -> Result<Self> {
        Self::load_ext(path, true).await
    }

    /// Sanitize the resource data
    fn sanitize(&mut self) {}

    /// Validates the resource after sanitization
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Saves the resource in same path as it was loaded
    async fn save(&self) -> Result<()> {
        let mut save_path = self.metadata().internal.path.to_path_buf();
        if save_path.is_dir() {
            tokio::fs::create_dir_all(&save_path).await?;
            save_path = search_resource_entrypoint(&save_path)
                .await
                .unwrap_or_else(|| save_path.join("metadata.yml"));
        }

        let extension = save_path
            .extension()
            .ok_or("Invalid path extension")?
            .to_string_lossy()
            .to_lowercase();

        match extension.as_str() {
            "slu" => {
                let mut slu_file = SluResourceFile::load(&save_path).await?;
                slu_file.data = serde_json::to_value(self)?.into();
                slu_file.store(&save_path).await?;
            }
            "yml" | "yaml" => {
                let content = serde_yaml::to_string(self)?;
                tokio::fs::write(save_path, content).await?;
            }
            "json" | "jsonc" => {
                let content = serde_json::to_string_pretty(self)?;
                tokio::fs::write(save_path, content).await?;
            }
            _ => {
                return Err("Unsupported path extension".into());
            }
        }
        Ok(())
    }

    async fn delete(&self) -> Result<()> {
        let path = self.metadata().internal.path.to_path_buf();
        if path.is_dir() {
            tokio::fs::remove_dir_all(path).await?;
        } else {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }
}
