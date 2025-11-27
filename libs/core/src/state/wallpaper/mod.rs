use std::path::Path;

use url::Url;

use crate::{
    error::Result,
    resource::{
        InternalResourceMetadata, ResourceKind, ResourceMetadata, ResourceText, SluResource,
        WallpaperId,
    },
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Wallpaper {
    pub id: WallpaperId,
    pub metadata: ResourceMetadata,
    pub r#type: WallpaperKind,
    pub url: Option<Url>,
    pub thumbnail_url: Option<Url>,
    pub filename: Option<String>,
    #[serde(alias = "thumbnail_filename")]
    pub thumbnail_filename: Option<String>,
    /// Only used if the wallpaper type is `Layered`.\
    /// Custom css that will be applied only on this wallpaper.
    pub css: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WallpaperKind {
    #[serde(alias = "image")]
    Image,
    #[serde(alias = "video")]
    Video,
    #[serde(alias = "layered")]
    Layered,
    /// used for wallpapers created before v2.4.9, will be changed on sanitization
    #[default]
    Unsupported,
}

impl SluResource for Wallpaper {
    const KIND: ResourceKind = ResourceKind::Wallpaper;

    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ResourceMetadata {
        &mut self.metadata
    }

    fn sanitize(&mut self) {
        // migration step for old wallpapers
        if WallpaperKind::Unsupported == self.r#type {
            if let Some(filename) = &self.filename {
                if Self::SUPPORTED_VIDEOS
                    .iter()
                    .any(|ext| filename.ends_with(ext))
                {
                    self.r#type = WallpaperKind::Video;
                }
                if Self::SUPPORTED_IMAGES
                    .iter()
                    .any(|ext| filename.ends_with(ext))
                {
                    self.r#type = WallpaperKind::Image;
                }
            }
        }
    }

    fn validate(&self) -> Result<()> {
        if self.r#type == WallpaperKind::Unsupported {
            return Err("Unsupported wallpaper extension".into());
        }
        Ok(())
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

        let r#type = if Self::SUPPORTED_IMAGES.contains(&ext.as_str()) {
            WallpaperKind::Image
        } else if Self::SUPPORTED_VIDEOS.contains(&ext.as_str()) {
            WallpaperKind::Video
        } else {
            WallpaperKind::Unsupported
        };

        let wallpaper = Self {
            id,
            metadata,
            r#type,
            url: None,
            thumbnail_url: None,
            filename: Some(filename.clone()),
            thumbnail_filename: if Self::SUPPORTED_IMAGES.contains(&ext.as_str()) {
                Some(filename)
            } else {
                None
            },
            css: None,
        };
        wallpaper.save()?;

        Ok(wallpaper)
    }
}
