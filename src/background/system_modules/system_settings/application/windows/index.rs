//! Searchable index of the Settings app pages and Control Panel applets, built from the
//! Windows own indexes (already localized to the user's display language).

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use seelen_core::system_state::{SystemSettingEntry, SystemSettingKind};
use windows::Win32::{
    Storage::EnhancedStorage::PKEY_InfoTipText,
    UI::Shell::{
        BHID_EnumItems, IEnumShellItems, IShellItem2, SIGDN_DESKTOPABSOLUTEPARSING,
        SIGDN_NORMALDISPLAY,
    },
};
use windows_core::Interface;

use crate::{
    error::Result,
    utils::constants::SEELEN_COMMON,
    windows_api::{Com, WindowsApi},
};

use super::index_schema::{ApplicationInformation, PcSettingsIndex};

/// Package resources (pri) of the Settings app, used to resolve the `@{windows?ms-resource://...}`
/// strings of its search index when the `windows` pseudo-package can't be resolved directly.
const SETTINGS_APP_PRI: &str =
    r"SystemResources\Windows.UI.SettingsAppThreshold\Windows.UI.SettingsAppThreshold.pri";
/// "All Control Panel Items" shell folder, with a fallback to the legacy control panel folder.
const CONTROL_PANEL_FOLDERS: [&str; 2] = [
    r"::{26EE0668-A00A-44D7-9371-BEB064C98683}\0",
    r"::{21EC2020-3AEA-1069-A2DD-08002B30309D}",
];
/// Icon fonts installed on the system as (family, file in `%WINDIR%\Fonts`), Segoe Fluent Icons is Windows 11 only.
const SYSTEM_GLYPH_FONTS: [(&str, &str); 2] = [
    ("Segoe Fluent Icons", "SegoeIcons.ttf"),
    ("Segoe MDL2 Assets", "segmdl2.ttf"),
];
/// `OpenInNewWindow` (arrow link), present on all [`SYSTEM_GLYPH_FONTS`].
const GENERIC_SETTING_GLYPH: &str = "\u{E8A7}";
/// Folder inside the app cache where private fonts are copied, to be reachable by the webviews.
const FONTS_CACHE_FOLDER: &str = "system_fonts";

/// Builds the whole index. Slow (reads big XML files and walks the control panel folder).
pub fn build_entries() -> Vec<SystemSettingEntry> {
    let mut entries = SystemSettingsIndex::enum_settings_pages().unwrap_or_else(|e| {
        log::error!("Failed to enumerate settings pages: {e:?}");
        Vec::new()
    });
    entries.extend(
        SystemSettingsIndex::enum_control_panel_items().unwrap_or_else(|e| {
            log::error!("Failed to enumerate control panel items: {e:?}");
            Vec::new()
        }),
    );
    log::trace!("System settings index built with {} entries", entries.len());
    entries
}

/// Namespace of the index readers.
struct SystemSettingsIndex;

impl SystemSettingsIndex {
    /// Reads `%WINDIR%\ImmersiveControlPanel\Settings\AllSystemSettings_{GUID}.xml`.
    fn enum_settings_pages() -> Result<Vec<SystemSettingEntry>> {
        let windir = SEELEN_COMMON.system_dir();
        let pri = windir.join(SETTINGS_APP_PRI);
        let pri = pri.to_string_lossy();

        let mut fonts = GlyphFonts::new(windir);
        let mut seen_files = HashSet::new();
        let mut seen_entries = HashSet::new();
        let mut entries = Vec::new();

        for file in std::fs::read_dir(windir.join(r"ImmersiveControlPanel\Settings"))? {
            let path = file?.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with("AllSystemSettings_") || !name.ends_with(".xml") {
                continue;
            }

            let content = std::fs::read_to_string(&path)?;
            let index: PcSettingsIndex =
                match quick_xml::de::from_str(content.trim_start_matches('\u{feff}')) {
                    Ok(index) => index,
                    Err(e) => {
                        log::warn!("Failed to parse {path:?}: {e}");
                        continue;
                    }
                };

            log::trace!("Reading {} settings from {path:?}", index.items.len());
            let mut unresolved = 0;
            for item in index.items {
                // some index files don't include the page ids, so only usable entries are marked as seen.
                let (Some(page), Some(info)) = (item.page(), &item.info) else {
                    continue;
                };

                // the same entries are repeated across the index files
                if !seen_files.insert(item.filename.clone()) {
                    continue;
                }

                let Some(title) = info
                    .description
                    .as_deref()
                    .and_then(|s| resolve_settings_string(s, &pri))
                else {
                    unresolved += 1;
                    continue;
                };

                let target = format!("ms-settings:{page}");
                if !seen_entries.insert((target.clone(), title.to_lowercase())) {
                    continue;
                }

                let keywords = [&info.high_keywords, &info.keywords]
                    .into_iter()
                    .flatten()
                    .filter_map(|s| resolve_settings_string(s, &pri))
                    .flat_map(|s| {
                        s.split(';')
                            .map(|k| k.trim().to_string())
                            .collect::<Vec<_>>()
                    })
                    .filter(|k| !k.is_empty())
                    .collect();

                let icon = item
                    .app_info
                    .as_ref()
                    .and_then(|info| fonts.resolve(info))
                    .unwrap_or_else(|| fonts.generic());

                entries.push(SystemSettingEntry {
                    kind: SystemSettingKind::Settings,
                    title,
                    description: None,
                    keywords,
                    target,
                    glyph: Some(icon.glyph),
                    font_family: Some(icon.font_family),
                    font_path: icon.font_path,
                });
            }

            if unresolved > 0 {
                log::trace!("{unresolved} settings titles could not be resolved in {path:?}");
            }
        }

        Ok(entries)
    }

    /// Enumerates the "All Control Panel Items" shell folder.
    fn enum_control_panel_items() -> Result<Vec<SystemSettingEntry>> {
        Com::run_with_context(|| {
            let folder = CONTROL_PANEL_FOLDERS
                .iter()
                .find_map(|f| WindowsApi::get_shell_item(Path::new(f)).ok())
                .ok_or("Control panel shell folder not found")?;

            let enumerator: IEnumShellItems =
                unsafe { folder.BindToHandler(None, &BHID_EnumItems)? };
            let mut entries = Vec::new();

            loop {
                let mut items = [None; 1];
                let mut fetched = 0u32;
                unsafe { enumerator.Next(&mut items, Some(&mut fetched))? };
                if fetched == 0 {
                    break;
                }

                let Some(item) = &items[0] else {
                    continue;
                };
                let item: IShellItem2 = item.cast()?;

                let title = unsafe { item.GetDisplayName(SIGDN_NORMALDISPLAY)?.to_string()? };
                let parsing_name = unsafe {
                    item.GetDisplayName(SIGDN_DESKTOPABSOLUTEPARSING)?
                        .to_string()?
                };
                let description = unsafe { item.GetString(&PKEY_InfoTipText) }
                    .ok()
                    .and_then(|s| unsafe { s.to_string() }.ok())
                    .filter(|s| !s.is_empty());

                entries.push(SystemSettingEntry {
                    kind: SystemSettingKind::ControlPanel,
                    title,
                    description,
                    keywords: Vec::new(),
                    target: format!("shell:{parsing_name}"),
                    glyph: None,
                    font_family: None,
                    font_path: None,
                });
            }

            Ok(entries)
        })
    }
}

/// Resolves the glyph fonts used by the Settings app search index.
struct GlyphFonts {
    windir: PathBuf,
    /// Family used for the generic glyph, the best one installed.
    generic_family: &'static str,
    /// Private font files already copied to the cache, by source path (`None` if the copy failed).
    copied: HashMap<String, Option<PathBuf>>,
}

impl GlyphFonts {
    fn new(windir: &Path) -> Self {
        let generic_family = SYSTEM_GLYPH_FONTS
            .iter()
            .find(|(_, file)| windir.join("Fonts").join(file).exists())
            .map_or(SYSTEM_GLYPH_FONTS[1].0, |(family, _)| family);

        Self {
            windir: windir.to_path_buf(),
            generic_family,
            copied: HashMap::new(),
        }
    }

    /// Same as the native search, entries using unavailable fonts or images get a generic glyph.
    fn generic(&self) -> GlyphIcon {
        GlyphIcon {
            glyph: GENERIC_SETTING_GLYPH.to_string(),
            font_family: self.generic_family.to_string(),
            font_path: None,
        }
    }

    fn resolve(&mut self, info: &ApplicationInformation) -> Option<GlyphIcon> {
        let glyph = info.glyph.clone().filter(|g| !g.is_empty())?;
        let font_family = info.font_family.clone()?;

        if SYSTEM_GLYPH_FONTS
            .iter()
            .any(|(family, _)| *family == font_family)
        {
            return Some(GlyphIcon {
                glyph,
                font_family,
                font_path: None,
            });
        }

        let source = info.font_file_path.as_ref()?;
        let font_path = self
            .copied
            .entry(source.clone())
            .or_insert_with(|| {
                let source = expand_windir(source, &self.windir);
                copy_font_to_cache(&source)
                    .inspect_err(|e| log::warn!("Failed to copy font {source:?}: {e:?}"))
                    .ok()
            })
            .clone()?;

        Some(GlyphIcon {
            glyph,
            font_family,
            font_path: Some(font_path),
        })
    }
}

struct GlyphIcon {
    glyph: String,
    font_family: String,
    font_path: Option<PathBuf>,
}

/// The webviews can't read from the Windows folder, so private fonts are served from the app cache.
fn copy_font_to_cache(source: &Path) -> Result<PathBuf> {
    let folder = SEELEN_COMMON.app_cache_dir().join(FONTS_CACHE_FOLDER);
    std::fs::create_dir_all(&folder)?;
    let target = folder.join(source.file_name().ok_or("Invalid font path")?);
    std::fs::copy(source, &target)?;
    Ok(target)
}

/// Expands the `%WINDIR%` variable used on the Settings app search index.
fn expand_windir(path: &str, windir: &Path) -> PathBuf {
    match path.get(..8) {
        Some(prefix) if prefix.eq_ignore_ascii_case("%WINDIR%") => {
            windir.join(path[8..].trim_start_matches('\\'))
        }
        _ => PathBuf::from(path),
    }
}

/// Resolves an indirect string of the Settings app search index, returns `None` if it can't be resolved.
fn resolve_settings_string(source: &str, settings_pri: &str) -> Option<String> {
    let source = source.trim();
    if !source.starts_with('@') {
        return (!source.is_empty()).then(|| source.to_string());
    }

    let resolved = WindowsApi::resolve_indirect_string(source)
        .ok()
        .filter(|s| !s.is_empty() && !s.starts_with("@{"))
        .or_else(|| {
            let uri = source.strip_prefix("@{windows?")?;
            WindowsApi::resolve_indirect_string(&format!("@{{{settings_pri}?{uri}")).ok()
        })?;

    let resolved = resolved.trim();
    (!resolved.is_empty() && !resolved.starts_with("@{")).then(|| resolved.to_string())
}
