use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine};
use seelen_core::{
    resource::{IconPackId, ResourceKind, SluResource},
    state::{Icon, IconPack, IconPackEntry},
};

use crate::{
    error::Result, resources::RESOURCES, state::application::FULL_STATE,
    utils::constants::SEELEN_COMMON,
};

const USER_PACK_ID: &str = "@user/custom-icons";
const USER_PACK_FOLDER: &str = "__user_custom_icons";

fn pack_dir() -> PathBuf {
    SEELEN_COMMON.user_icons_path().join(USER_PACK_FOLDER)
}

fn sanitize_slug(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn slug_from_entry(entry: &IconPackEntry) -> String {
    match entry {
        IconPackEntry::Unique(u) => {
            if let Some(umid) = &u.umid {
                sanitize_slug(umid)
            } else if let Some(path) = &u.path {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(sanitize_slug)
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                "unknown".to_string()
            }
        }
        IconPackEntry::Shared(s) => format!("ext_{}", s.extension),
        IconPackEntry::Custom(c) => sanitize_slug(&c.key),
    }
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

    let slug = slug_from_entry(&entry);
    let filename = format!("{slug}.png");
    let bytes = STANDARD.decode(&icon_base64)?;
    std::fs::write(icon_dir.join(&filename), bytes)?;

    let mut pack = load_or_create_pack(&dir);
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
