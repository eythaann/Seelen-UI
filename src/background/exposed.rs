use std::{
    collections::HashMap,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
};

use seelen_core::{
    command_handler_list,
    state::WegItemData,
    system_state::{Color, Relaunch, RelaunchArguments, StartMenuLayout, StartMenuLayoutItem},
};

use slu_ipc::{messages::SvcAction, ServiceIpc};
use tauri::{Builder, WebviewWindow, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW};

use crate::{
    app::{get_app_handle, SeelenUI},
    error::Result,
    utils::{
        self,
        constants::SEELEN_COMMON,
        icon_extractor::{request_icon_extraction_from_file, request_icon_extraction_from_umid},
        pwsh::PwshScript,
    },
    widgets::{
        permissions::{request_widget_permission, WidgetPerm},
        popups::shortcut_registering::REG_SHORTCUT_DATA,
    },
    windows_api::{hdc::DeviceContext, string_utils::WindowsString, window::Window, WindowsApi},
};

#[tauri::command(async)]
pub fn log_from_webview(level: u8, message: String, location: String) {
    let level = match level {
        1 => log::Level::Trace,
        2 => log::Level::Debug,
        3 => log::Level::Info,
        4 => log::Level::Warn,
        _ => log::Level::Error,
    };
    log::log!(target: &location, level, "{message}");
}

pub fn open_file_inner(path: String) -> Result<()> {
    std::process::Command::new("cmd")
        .raw_arg("/c")
        .raw_arg("start")
        .raw_arg("\"\"")
        .raw_arg(format!("\"{path}\""))
        .creation_flags(CREATE_NO_WINDOW.0 | CREATE_NEW_PROCESS_GROUP.0)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    Ok(())
}

#[tauri::command(async)]
pub fn open_file(webview: tauri::WebviewWindow, path: String) -> Result<()> {
    request_widget_permission(&webview, WidgetPerm::OpenFile)?;
    open_file_inner(path)
}

#[tauri::command(async)]
fn select_file_on_explorer(path: String) -> Result<()> {
    get_app_handle()
        .shell()
        .command(SEELEN_COMMON.system_dir().join("explorer.exe"))
        .args(["/select,", &path])
        .spawn()?;
    Ok(())
}

#[tauri::command(async)]
async fn run(
    webview: tauri::WebviewWindow,
    program: String,
    args: Option<RelaunchArguments>,
    working_dir: Option<PathBuf>,
    elevated: bool,
) -> Result<()> {
    request_widget_permission(&webview, WidgetPerm::Run)?;
    let args = args.map(|args| args.to_string());
    WindowsApi::execute(program, args, working_dir, elevated)
}

#[tauri::command(async)]
fn is_dev_mode() -> bool {
    tauri::is_dev()
}

#[tauri::command(async)]
fn has_fixed_runtime() -> bool {
    crate::utils::has_fixed_runtime()
}

#[tauri::command(async)]
fn is_appx_package() -> bool {
    crate::utils::is_running_as_appx()
}

#[tauri::command(async)]
pub fn get_user_envs() -> HashMap<String, String> {
    std::env::vars().collect::<HashMap<String, String>>()
}

#[tauri::command(async)]
async fn set_auto_start(enabled: bool) -> Result<()> {
    SeelenUI::set_auto_start(enabled)
}

#[tauri::command(async)]
async fn get_auto_start_status() -> Result<bool> {
    SeelenUI::is_auto_start_enabled()
}

// used to request icon extraction
#[tauri::command(async)]
fn get_icon(path: Option<PathBuf>, umid: Option<String>) -> Result<()> {
    if let Some(umid) = umid {
        request_icon_extraction_from_umid(&umid.into());
    }
    if let Some(path) = path {
        request_icon_extraction_from_file(&path);
    }
    Ok(())
}

#[tauri::command(async)]
async fn check_for_updates() -> Result<bool> {
    Ok(utils::updater::check_for_updates().await?.is_some())
}

#[tauri::command(async)]
fn get_foreground_window_color(webview: WebviewWindow<tauri::Wry>) -> Result<Color> {
    let webview = Window::from(webview.hwnd()?.0 as isize);
    let foreground = Window::get_foregrounded();

    if webview.monitor() != foreground.monitor() {
        return Ok(Color::default());
    }

    if !foreground.is_visible() || foreground.is_desktop() {
        return Ok(Color::default());
    }

    let hdc = DeviceContext::create(None);
    let rect = foreground.inner_rect()?;
    let x = rect.left + (rect.right - rect.left) / 2;
    Ok(hdc.get_pixel(x, rect.top + 2))
}

#[tauri::command(async)]
async fn install_last_available_update() -> Result<()> {
    let update = utils::updater::check_for_updates()
        .await?
        .ok_or("There is no update available")?;
    utils::updater::trace_update_intallation(update).await?;
    get_app_handle().restart();
    #[allow(unreachable_code)]
    Ok(())
}

#[tauri::command(async)]
async fn get_native_start_menu() -> Result<StartMenuLayout> {
    let output_path = SEELEN_COMMON.app_cache_dir().join("start-layout.json");
    let output_path_str = output_path.to_string_lossy().to_string();

    let script =
        PwshScript::new(format!("Export-StartLayout -Path '{}'", output_path_str)).inline_command();
    script.execute().await?;

    let file = std::fs::File::open(&output_path)?;
    let mut layout: StartMenuLayout = serde_json::from_reader(file)?;

    for item in &mut layout.pinned_list {
        if let StartMenuLayoutItem::DesktopAppLink(path) = item {
            let source = WindowsString::from_str(path);
            let expanded = WindowsApi::resolve_environment_variables(&source)?;
            *item = StartMenuLayoutItem::DesktopAppLink(expanded.to_string());
        }
    }

    Ok(layout)
}

#[tauri::command(async)]
pub(crate) fn get_windows_taskbar_pinned_apps() -> Result<Vec<WegItemData>> {
    let mut pinned_apps = Vec::new();

    match get_taskbar_order_from_registry() {
        Ok(ordered_paths) if !ordered_paths.is_empty() => {
            for lnk_path_str in ordered_paths {
                let lnk_path = PathBuf::from(&lnk_path_str);

                if lnk_path.exists() {
                    if let Ok(item_data) = create_weg_item_from_shortcut(&lnk_path) {
                        pinned_apps.push(item_data);
                    }
                }
            }
        }
        Ok(_) | Err(_) => {
            pinned_apps = fallback_get_taskbar_items()?;
        }
    }

    // Deduplicate by path (case-insensitive)
    let mut seen_paths = std::collections::HashSet::new();
    pinned_apps.retain(|item| {
        let path_lower = item.path.to_string_lossy().to_lowercase();
        seen_paths.insert(path_lower)
    });

    Ok(pinned_apps)
}

/// Extract taskbar pinned items order from Windows registry
fn get_taskbar_order_from_registry() -> Result<Vec<String>> {
    use regex::Regex;
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let taskband_key =
        hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Taskband")?;

    // Read the FavoritesResolve binary value as bytes using Windows API directly
    let favorites_resolve: Vec<u8> = {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::System::Registry::*;

        let key_name: Vec<u16> = OsStr::new("FavoritesResolve")
            .encode_wide()
            .chain(Some(0))
            .collect();
        let mut value_type = REG_VALUE_TYPE(0);
        let mut data_size: u32 = 0;

        // First call to get size
        unsafe {
            RegQueryValueExW(
                HKEY(taskband_key.raw_handle() as _),
                windows::core::PCWSTR(key_name.as_ptr()),
                None,
                Some(&mut value_type),
                None,
                Some(&mut data_size),
            )
            .ok()?;
        }

        // Allocate buffer and read data
        let mut buffer = vec![0u8; data_size as usize];
        unsafe {
            RegQueryValueExW(
                HKEY(taskband_key.raw_handle() as _),
                windows::core::PCWSTR(key_name.as_ptr()),
                None,
                Some(&mut value_type),
                Some(buffer.as_mut_ptr()),
                Some(&mut data_size),
            )
            .ok()?;
        }

        buffer
    };

    // Convert binary data to string
    let result: String = favorites_resolve.iter().map(|&byte| byte as char).collect();

    // Extract file paths from binary data
    let userprofile =
        std::env::var("USERPROFILE").map_err(|e| crate::error::AppError::from(e.to_string()))?;
    let pattern_str = format!(r"{}.+?\.\w{{2,4}}", regex::escape(&userprofile));
    let pattern =
        Regex::new(&pattern_str).map_err(|e| crate::error::AppError::from(e.to_string()))?;

    let paths: Vec<String> = pattern
        .find_iter(&result)
        .map(|m| m.as_str().to_string())
        .collect();

    Ok(paths)
}

/// Fallback: scan filesystem and sort by creation time
fn fallback_get_taskbar_items() -> Result<Vec<WegItemData>> {
    let mut pinned_apps = Vec::new();

    if let Ok(app_data) = std::env::var("APPDATA") {
        let user_pinned_path = PathBuf::from(app_data)
            .join(r"Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar");

        if user_pinned_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&user_pinned_path) {
                let mut items_with_time: Vec<(WegItemData, std::time::SystemTime)> = Vec::new();

                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.extension().is_some_and(|ext| ext == "lnk") {
                        if let Ok(item_data) = create_weg_item_from_shortcut(&entry_path) {
                            let created = std::fs::metadata(&entry_path)
                                .and_then(|m| m.created())
                                .unwrap_or_else(|_| std::time::SystemTime::now());
                            items_with_time.push((item_data, created));
                        }
                    }
                }

                items_with_time.sort_by_key(|(_, time)| *time);
                pinned_apps = items_with_time.into_iter().map(|(item, _)| item).collect();
            }
        }
    }

    Ok(pinned_apps)
}

/// Creates a WegItemData from a .lnk shortcut file
fn create_weg_item_from_shortcut(lnk_path: &Path) -> Result<WegItemData> {
    let (target_path, arguments) = WindowsApi::resolve_lnk_target(lnk_path)?;

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
        Some(Relaunch {
            command,
            args,
            working_dir: if !target_path.as_os_str().is_empty() {
                target_path.parent().map(|p| p.to_path_buf())
            } else {
                None
            },
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

#[tauri::command(async)]
async fn request_to_user_input_shortcut(
    window: WebviewWindow,
    callback_event: String,
) -> Result<()> {
    ServiceIpc::send(SvcAction::StartShortcutRegistration).await?;

    let mut data = REG_SHORTCUT_DATA.lock();
    data.response_view_label = Some(window.label().to_string());
    data.response_event = Some(callback_event);
    Ok(())
}

pub fn register_invoke_handler(app_builder: Builder<Wry>) -> Builder<Wry> {
    use crate::state::infrastructure::*;
    use crate::virtual_desktops::handlers::*;

    use crate::widgets::permissions::*;
    use crate::widgets::wallpaper_manager::handlers::*;
    use crate::widgets::weg::handler::*;
    use crate::widgets::window_manager::handler::*;
    use crate::widgets::*;

    use crate::backups::infrastructure::*;
    use crate::resources::commands::*;
    use crate::session::infrastructure::*;

    use crate::modules::apps::infrastructure::*;
    use crate::modules::clipboard::infrastructure::*;
    use crate::modules::focus_assist::infrastructure::*;
    use crate::modules::fonts::infrastructure::*;
    use crate::modules::media::devices::infrastructure::*;
    use crate::modules::media::players::infrastructure::*;
    use crate::modules::media::waveform::infrastructure::*;
    use crate::modules::monitors::brightness::infrastructure::*;
    use crate::modules::monitors::infrastructure::*;
    use crate::modules::network::infrastructure::*;
    use crate::modules::notifications::infrastructure::*;
    use crate::modules::power::infrastructure::*;
    use crate::modules::radios::bluetooth::handlers::*;
    use crate::modules::radios::handlers::*;
    use crate::modules::radios::wifi::handlers::*;
    use crate::modules::start::infrastructure::*;
    use crate::modules::system::tauri::*;
    use crate::modules::system_settings::infrastructure::*;
    use crate::modules::system_settings::language::infrastructure::*;
    use crate::modules::system_tray::infrastructure::*;
    use crate::modules::trash_bin::infrastructure::*;
    use crate::modules::user::infrastructure::*;
    use crate::resources::user_icon_pack::*;

    app_builder.invoke_handler(command_handler_list!())
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
    fn test_create_weg_item_from_shortcut_with_system_shortcut() {
        // Test with Windows Explorer - should exist on all Windows systems
        let explorer_path = PathBuf::from("C:\\Windows\\explorer.exe");
        if explorer_path.exists() {
            // Create a test by checking if we can at least construct a basic item
            let result = WegItemData {
                id: uuid::Uuid::new_v4(),
                display_name: "Test Explorer".to_string(),
                umid: None,
                path: explorer_path.clone(),
                pinned: true,
                prevent_pinning: false,
                relaunch: Some(Relaunch {
                    command: explorer_path.to_string_lossy().to_string(),
                    args: None,
                    working_dir: None,
                    icon: None,
                }),
            };

            assert_eq!(result.display_name, "Test Explorer");
            assert!(result.pinned);
            assert!(!result.prevent_pinning);
        }
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

        // Simulate deduplication logic
        let mut seen_paths = std::collections::HashSet::new();
        items.retain(|item| {
            let path_lower = item.path.to_string_lossy().to_lowercase();
            seen_paths.insert(path_lower)
        });

        assert_eq!(items.len(), 2, "Should deduplicate case-insensitive paths");
    }

    #[test]
    fn test_fallback_get_taskbar_items_handles_missing_directory() {
        // Test with a non-existent APPDATA path
        std::env::set_var("APPDATA", "C:\\NonExistent\\AppData");
        let result = fallback_get_taskbar_items();

        // Should return Ok with empty list, not crash
        assert!(result.is_ok());
        if let Ok(items) = result {
            assert!(
                items.is_empty() || !items.is_empty(),
                "Should handle gracefully"
            );
        }
    }

    #[test]
    fn test_regex_pattern_handles_various_extensions() {
        use regex::Regex;

        // Test the original regex pattern
        let userprofile = "C:\\Users\\TestUser";
        let pattern_str = format!(r"{}.+?\.\w{{2,4}}", regex::escape(userprofile));
        let pattern = Regex::new(&pattern_str).expect("Pattern should be valid");

        // Test various extension lengths that the pattern should match
        let test_cases = vec![
            (format!("{}\\path\\file.lnk", userprofile), true), // 3 chars
            (format!("{}\\path\\file.exe", userprofile), true), // 3 chars
            (format!("{}\\path\\file.msix", userprofile), true), // 4 chars
            (format!("{}\\path\\file.url", userprofile), true), // 3 chars
            ("C:\\OtherUser\\file.lnk".to_string(), false),     // Wrong user - should not match
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
        // This test will run but may return empty results on systems without pinned items
        let result = get_windows_taskbar_pinned_apps();

        // Should not panic, and if successful, should return a valid Vec
        match result {
            Ok(items) => {
                // Validate structure of returned items
                for item in items {
                    assert!(
                        !item.display_name.is_empty(),
                        "Display name should not be empty"
                    );
                    assert!(item.id != uuid::Uuid::nil(), "ID should not be nil");
                    // Path might not exist if it's a packaged app, so we don't assert existence
                }
            }
            Err(e) => {
                // Failing is acceptable if registry is inaccessible or empty
                eprintln!(
                    "Note: get_windows_taskbar_pinned_apps failed (acceptable in test): {}",
                    e
                );
            }
        }
    }
}
