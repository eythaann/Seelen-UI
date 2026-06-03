use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine};
use seelen_core::{
    resource::{IconPackId, ResourceKind, SluResource},
    state::{Icon, IconPack, IconPackEntry},
};

use crate::{
    error::Result,
    resources::RESOURCES,
    state::application::FULL_STATE,
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
};

const USER_PACK_ID: &str = "@user/custom-icons";
const USER_PACK_FOLDER: &str = "__user_custom_icons";

fn pack_dir() -> PathBuf {
    SEELEN_COMMON.user_icons_path().join(USER_PACK_FOLDER)
}

fn with_icon(entry: IconPackEntry, icon_rel_path: String) -> IconPackEntry {
    match entry {
        IconPackEntry::Unique(mut u) => {
            let existing = u.icon.unwrap_or_default();
            u.icon = Some(Icon {
                base: Some(icon_rel_path),
                light: None,
                dark: None,
                mask: None,
                is_aproximately_square: existing.is_aproximately_square,
            });
            u.redirect = None;
            IconPackEntry::Unique(u)
        }
        IconPackEntry::Shared(mut s) => {
            s.icon = Icon {
                base: Some(icon_rel_path),
                light: None,
                dark: None,
                mask: None,
                is_aproximately_square: s.icon.is_aproximately_square,
            };
            IconPackEntry::Shared(s)
        }
        IconPackEntry::Custom(mut c) => {
            c.icon = Icon {
                base: Some(icon_rel_path),
                light: None,
                dark: None,
                mask: None,
                is_aproximately_square: c.icon.is_aproximately_square,
            };
            IconPackEntry::Custom(c)
        }
    }
}

fn load_or_create_pack(dir: &Path) -> IconPack {
    if dir.exists() {
        if let Ok(pack) = IconPack::load(dir) {
            return pack;
        }
    }
    let mut pack = IconPack {
        id: USER_PACK_ID.into(),
        ..Default::default()
    };
    pack.metadata.internal.path = dir.to_path_buf();
    pack
}

#[tauri::command(async)]
pub fn register_user_custom_app_icon(icon_base64: String, entry: IconPackEntry) -> Result<()> {
    let dir = pack_dir();
    let icon_dir = dir.join("icons");
    std::fs::create_dir_all(&icon_dir)?;

    let mut pack = load_or_create_pack(&dir);

    // Delete the old icon file on disk when replacing an existing entry
    let old_rel = pack
        .find_similar(&entry)
        .and_then(|existing| match existing {
            IconPackEntry::Unique(u) => u.icon.as_ref().and_then(|i| i.base.clone()),
            IconPackEntry::Shared(s) => s.icon.base.clone(),
            IconPackEntry::Custom(c) => c.icon.base.clone(),
        });
    if let Some(rel) = old_rel {
        let _ = std::fs::remove_file(dir.join(rel));
    }

    let filename = format!("{}.png", date_based_hex_id());
    let bytes = STANDARD.decode(&icon_base64)?;
    std::fs::write(icon_dir.join(&filename), bytes)?;

    pack.add_entry(with_icon(entry, format!("icons/{filename}")));
    pack.save()?;

    RESOURCES.load(&ResourceKind::IconPack, &dir)?;

    let pack_id: IconPackId = USER_PACK_ID.into();
    FULL_STATE.rcu(|state| {
        let mut state = state.cloned();
        state.settings.active_icon_packs.retain(|id| id != &pack_id);
        state.settings.active_icon_packs.push(pack_id.clone());
        state
    });
    FULL_STATE.load().write_settings()?;

    RESOURCES.emit_icon_packs();
    Ok(())
}

#[tauri::command(async)]
pub fn delete_user_custom_app_icon(entry: IconPackEntry) -> Result<()> {
    let dir = pack_dir();
    if !dir.exists() {
        return Ok(());
    }

    let mut pack = load_or_create_pack(&dir);

    let old_rel = pack
        .find_similar(&entry)
        .and_then(|existing| match existing {
            IconPackEntry::Unique(u) => u.icon.as_ref().and_then(|i| i.base.clone()),
            IconPackEntry::Shared(s) => s.icon.base.clone(),
            IconPackEntry::Custom(c) => c.icon.base.clone(),
        });
    if let Some(rel) = old_rel {
        let _ = std::fs::remove_file(dir.join(rel));
    }

    pack.entries.retain(|e| !e.matches(&entry));
    pack.save()?;

    RESOURCES.load(&ResourceKind::IconPack, &dir)?;
    RESOURCES.emit_icon_packs();
    Ok(())
}
