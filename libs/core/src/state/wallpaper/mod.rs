use std::{fs::File, path::Path};

use url::Url;

use crate::{
    error::Result,
    resource::{
        ConcreteResource, InternalResourceMetadata, ResourceMetadata, ResourceText, SluResource,
        SluResourceFile, WallpaperId,
    },
    utils::search_for_metadata_file,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Wallpaper {
    pub id: WallpaperId,
    pub metadata: ResourceMetadata,
    pub url: Option<Url>,
    pub thumbnail_url: Option<Url>,
    pub filename: Option<String>,
    pub thumbnail_filename: Option<String>,
}

impl SluResource for Wallpaper {
    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ResourceMetadata {
        &mut self.metadata
    }

    fn load_from_file(path: &Path) -> Result<Self> {
        let extension = path
            .extension()
            .ok_or("Invalid theme path extension")?
            .to_string_lossy();

        let wallpaper = match extension.as_ref() {
            "yml" | "yaml" => serde_yaml::from_reader(File::open(path)?)?,
            "json" | "jsonc" => serde_json::from_reader(File::open(path)?)?,
            "slu" => match SluResourceFile::load(path)?.concrete()? {
                ConcreteResource::Wallpaper(parsed) => parsed,
                _ => return Err("Resource file is not a wallpaper".into()),
            },
            _ => {
                return Err("Invalid wallpaper path extension".into());
            }
        };
        Ok(wallpaper)
    }

    fn load_from_folder(path: &Path) -> Result<Self> {
        let metadata = search_for_metadata_file(path).ok_or("No metadata file found")?;
        Self::load_from_file(&metadata)
    }
}

impl Wallpaper {
    /// https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Formats/Image_types
    pub const SUPPORTED_IMAGES: [&str; 11] = [
        "apng", "avif", "gif", "jpg", "jpeg", "png", "svg", "webp", "bmp", "ico", "tiff",
    ];
    /// https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Formats/Containers
    pub const SUPPORTED_VIDEOS: [&str; 7] = ["mp4", "webm", "ogg", "avi", "mov", "mkv", "mpeg"];

    /// path should be the path to the wallpaper image or video to be moved or copied to the wallpaper folder
    pub fn create_from_file(path: &Path, folder_to_store: &Path, copy: bool) -> Result<Self> {
        if !path.exists() || path.is_dir() {
            return Err("File does not exist".into());
        }

        let (Some(filename), Some(ext)) = (path.file_name(), path.extension()) else {
            return Err("Invalid file name or extension".into());
        };
        let filename = filename.to_string_lossy().to_string();
        let ext = ext.to_string_lossy().to_string();

        // as uuids can start with numbers and resources names can't start with numbers
        // we prefix the uuid with an 'x'
        let resource_name = uuid::Uuid::new_v4();
        let id = format!("@user/x{}", resource_name.as_simple()).into();

        let metadata = ResourceMetadata {
            display_name: ResourceText::En(filename.clone()),
            internal: InternalResourceMetadata {
                path: folder_to_store.join("metadata.yml"),
                ..Default::default()
            },
            ..Default::default()
        };

        std::fs::create_dir_all(folder_to_store)?;
        if copy {
            std::fs::copy(path, folder_to_store.join(&filename))?;
        } else {
            std::fs::rename(path, folder_to_store.join(&filename))?;
        }

        let wallpaper = Self {
            id,
            metadata,
            filename: Some(filename.clone()),
            thumbnail_filename: if Self::SUPPORTED_IMAGES.contains(&ext.as_str()) {
                Some(filename)
            } else {
                None
            },
            ..Default::default()
        };
        wallpaper.save()?;

        Ok(wallpaper)
    }
}
