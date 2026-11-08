use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use seelen_core::state::WegItemData;
use seelen_core::system_state::RelaunchArguments;
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

use crate::{error::Result, windows_api::WindowsApi};

/// Extracts the list of apps currently pinned to the native Windows taskbar.
///
/// It first tries to read the pin order from the registry (`Taskband\FavoritesResolve`),
/// falling back to scanning the "Quick Launch\User Pinned\TaskBar" folder (sorted by
/// creation time) when the registry value is missing, empty or unreadable.
pub fn get_windows_taskbar_pinned_apps() -> Result<Vec<WegItemData>> {
    let mut pinned_apps = match get_taskbar_order_from_registry() {
        Ok(ordered_paths) if !ordered_paths.is_empty() => ordered_paths
            .into_iter()
            .map(PathBuf::from)
            .filter(|lnk_path| lnk_path.exists())
            .filter_map(|lnk_path| create_weg_item_from_shortcut(&lnk_path).ok())
            .collect(),
        _ => fallback_get_taskbar_items()?,
    };

    // Deduplicate by path (case-insensitive)
    let mut seen_paths = HashSet::new();
    pinned_apps.retain(|item: &WegItemData| {
        let path_lower = item.path.to_string_lossy().to_lowercase();
        seen_paths.insert(path_lower)
    });

    Ok(pinned_apps)
}

/// Extract taskbar pinned items order from the Windows registry.
///
/// Windows stores the pinned taskbar order as a binary blob (`FavoritesResolve`) under
/// `HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband`. That blob embeds the
/// `.lnk` shortcut paths as UTF-16-ish ASCII runs, so we scan the raw bytes for anything that
/// looks like a path under the user's profile.
fn get_taskbar_order_from_registry() -> Result<Vec<String>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let taskband_key =
        hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Taskband")?;

    let favorites_resolve = taskband_key.get_raw_value("FavoritesResolve")?.bytes;
    let result: String = favorites_resolve.iter().map(|&byte| byte as char).collect();

    let userprofile =
        std::env::var("USERPROFILE").map_err(|e| crate::error::AppError::from(e.to_string()))?;
    let pattern_str = format!(r"{}.+?\.\w{{2,4}}", regex::escape(&userprofile));
    let pattern =
        regex::Regex::new(&pattern_str).map_err(|e| crate::error::AppError::from(e.to_string()))?;

    let paths = pattern
        .find_iter(&result)
        .map(|m| m.as_str().to_string())
        .collect();

    Ok(paths)
}

/// Fallback: scan the Quick Launch pinned taskbar folder and sort by creation time.
fn fallback_get_taskbar_items() -> Result<Vec<WegItemData>> {
    let mut pinned_apps = Vec::new();

    if let Ok(app_data) = std::env::var("APPDATA") {
        let user_pinned_path = PathBuf::from(app_data)
            .join(r"Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar");

        if let Ok(entries) = std::fs::read_dir(&user_pinned_path) {
            let mut items_with_time: Vec<(WegItemData, std::time::SystemTime)> = Vec::new();

            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.extension().is_some_and(|ext| ext == "lnk")
                    && let Ok(item_data) = create_weg_item_from_shortcut(&entry_path)
                {
                    let created = std::fs::metadata(&entry_path)
                        .and_then(|m| m.created())
                        .unwrap_or_else(|_| std::time::SystemTime::now());
                    items_with_time.push((item_data, created));
                }
            }

            items_with_time.sort_by_key(|(_, time)| *time);
            pinned_apps = items_with_time.into_iter().map(|(item, _)| item).collect();
        }
    }

    Ok(pinned_apps)
}

/// Creates a `WegItemData` from a `.lnk` shortcut file.
fn create_weg_item_from_shortcut(lnk_path: &Path) -> Result<WegItemData> {
    let (target_path, arguments, working_dir) = WindowsApi::resolve_lnk_target(lnk_path)?;

    let display_name = lnk_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let umid = WindowsApi::get_file_umid(lnk_path).ok();

    let args = if arguments.is_empty() {
        None
    } else {
        Some(RelaunchArguments::String(
            arguments.to_string_lossy().to_string(),
        ))
    };

    let custom_icon_path = WindowsApi::resolve_lnk_custom_icon_path(lnk_path)
        .ok()
        .map(|(icon_path, _)| icon_path);

    let command = if !target_path.as_os_str().is_empty() {
        target_path.to_string_lossy().to_string()
    } else if let Some(ref icon_path) = custom_icon_path {
        icon_path.to_string_lossy().to_string()
    } else {
        String::new()
    };

    let relaunch = if !command.is_empty() {
        let working_dir = if !working_dir.as_os_str().is_empty() {
            Some(working_dir)
        } else {
            target_path.parent().map(|p| p.to_path_buf())
        };

        Some(seelen_core::system_state::Relaunch {
            command,
            args,
            working_dir,
            icon: None,
        })
    } else {
        None
    };

    Ok(WegItemData {
        id: uuid::Uuid::new_v4(),
        display_name,
        umid,
        path: lnk_path.to_path_buf(),
        pinned: true,
        prevent_pinning: false,
        relaunch,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_weg_item_from_shortcut_handles_invalid_path() {
        let invalid_path = PathBuf::from("C:\\NonExistent\\Path\\file.lnk");
        let result = create_weg_item_from_shortcut(&invalid_path);
        assert!(result.is_err(), "Should fail for non-existent path");
    }

    #[test]
    fn test_deduplication_case_insensitive() {
        let mut items = vec![
            WegItemData {
                id: uuid::Uuid::new_v4(),
                display_name: "Item1".to_string(),
                umid: None,
                path: PathBuf::from("C:\\Test\\App.exe"),
                pinned: true,
                prevent_pinning: false,
                relaunch: None,
            },
            WegItemData {
                id: uuid::Uuid::new_v4(),
                display_name: "Item2".to_string(),
                umid: None,
                path: PathBuf::from("C:\\TEST\\APP.EXE"), // Same path, different case
                pinned: true,
                prevent_pinning: false,
                relaunch: None,
            },
            WegItemData {
                id: uuid::Uuid::new_v4(),
                display_name: "Item3".to_string(),
                umid: None,
                path: PathBuf::from("C:\\Other\\App.exe"),
                pinned: true,
                prevent_pinning: false,
                relaunch: None,
            },
        ];

        let mut seen_paths = HashSet::new();
        items.retain(|item| {
            let path_lower = item.path.to_string_lossy().to_lowercase();
            seen_paths.insert(path_lower)
        });

        assert_eq!(items.len(), 2, "Should deduplicate case-insensitive paths");
    }

    #[test]
    fn test_fallback_get_taskbar_items_handles_missing_directory() {
        unsafe { std::env::set_var("APPDATA", "C:\\NonExistent\\AppData") };
        let result = fallback_get_taskbar_items();

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_regex_pattern_handles_various_extensions() {
        let userprofile = "C:\\Users\\TestUser";
        let pattern_str = format!(r"{}.+?\.\w{{2,4}}", regex::escape(userprofile));
        let pattern = regex::Regex::new(&pattern_str).expect("Pattern should be valid");

        let test_cases = vec![
            (format!("{}\\path\\file.lnk", userprofile), true),
            (format!("{}\\path\\file.exe", userprofile), true),
            (format!("{}\\path\\file.msix", userprofile), true),
            (format!("{}\\path\\file.url", userprofile), true),
            ("C:\\OtherUser\\file.lnk".to_string(), false),
        ];

        for (path, should_match) in test_cases {
            let matches = pattern.is_match(&path);
            assert_eq!(
                matches, should_match,
                "Pattern matching failed for: {}. Expected: {}, Got: {}",
                path, should_match, matches
            );
        }
    }

    #[test]
    fn test_get_windows_taskbar_pinned_apps_returns_valid_structure() {
        let result = get_windows_taskbar_pinned_apps();

        match result {
            Ok(items) => {
                for item in items {
                    assert!(
                        !item.display_name.is_empty(),
                        "Display name should not be empty"
                    );
                    assert!(item.id != uuid::Uuid::nil(), "ID should not be nil");
                }
            }
            Err(e) => {
                eprintln!(
                    "Note: get_windows_taskbar_pinned_apps failed (acceptable in test): {}",
                    e
                );
            }
        }
    }
}
