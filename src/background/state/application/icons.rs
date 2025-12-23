use std::path::{Path, PathBuf};

use seelen_core::{
    resource::SluResource,
    state::{Icon, IconPack, IconPackEntry},
};

use crate::{error::Result, utils::date_based_hex_id};

pub async fn download_remote_icons(pack: &mut IconPack) -> Result<()> {
    if pack.remote_entries.is_empty() || pack.downloaded {
        return Ok(());
    }

    let folder_to_store = pack.metadata.directory()?;
    let mut entries = Vec::new();

    for entry in &pack.remote_entries {
        let mut new_entry = entry.clone();

        match &mut new_entry {
            IconPackEntry::Unique(entry) => {
                if let Some(icon) = &mut entry.icon {
                    *icon = download_entry_icons(icon, &folder_to_store).await?;
                }
            }
            IconPackEntry::Shared(entry) => {
                entry.icon = download_entry_icons(&entry.icon, &folder_to_store).await?;
            }
            IconPackEntry::Custom(entry) => {
                entry.icon = download_entry_icons(&entry.icon, &folder_to_store).await?;
            }
        }

        entries.push(new_entry);
    }

    pack.entries = entries;
    pack.downloaded = true;
    pack.save()?;
    Ok(())
}

// download remote icon url and save it on the parent path + random hash.
async fn download_entry_icons(icon: &Icon, folder_to_store: &Path) -> Result<Icon> {
    let mut resolved = icon.clone();

    let download_filename = async |url: &str| -> Result<String> {
        Ok(download_remote_icon_and_validate_it(url, folder_to_store)
            .await?
            .file_name()
            .ok_or("Could not get file name")?
            .to_string_lossy()
            .to_string())
    };

    if let Some(url) = &icon.base {
        resolved.base = download_filename(url).await.ok();
    }

    if let Some(url) = &icon.light {
        resolved.light = download_filename(url).await.ok();
    }

    if let Some(url) = &icon.dark {
        resolved.dark = download_filename(url).await.ok()
    }

    if let Some(url) = &icon.mask {
        resolved.mask = download_filename(url).await.ok();
    }

    Ok(resolved)
}

/// returns a path to the downloaded icon
async fn download_remote_icon_and_validate_it(
    url: &str,
    folder_to_store: &Path,
) -> Result<PathBuf> {
    if !folder_to_store.is_dir() {
        return Err("Folder to store is not a directory".into());
    }

    let res = reqwest::get(url).await?;
    let bytes = res.bytes().await?;

    let format = image::guess_format(&bytes)?;
    let icon = image::load_from_memory_with_format(&bytes, format)?;
    let extension = format
        .extensions_str()
        .first()
        .ok_or("Could not get extension")?;

    let icon_path = folder_to_store.join(format!("{}.{}", date_based_hex_id(), extension));
    icon.save(&icon_path)?;
    Ok(icon_path)
}
