use std::collections::HashMap;
use std::path::PathBuf;

use seelen_core::{command_handler_list, system_state::Color};

use tauri::{Builder, WebviewWindow, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::Foundation::HWND;

use crate::error_handler::Result;
use crate::hook::HookManager;
use crate::modules::input::Keyboard;
use crate::modules::virtual_desk::get_vd_manager;
use crate::seelen::{get_app_handle, Seelen};

use crate::seelen_weg::icon_extractor::{
    extract_and_save_icon_from_file, extract_and_save_icon_umid,
};

use crate::utils::pwsh::PwshScript;
use crate::utils::{is_running_as_appx, is_virtual_desktop_supported as virtual_desktop_supported};
use crate::windows_api::hdc::DeviceContext;
use crate::windows_api::window::Window;
use crate::windows_api::WindowsApi;
use crate::winevent::{SyntheticFullscreenData, WinEvent};
use crate::{log_error, utils};

#[tauri::command(async)]
fn select_file_on_explorer(path: String) -> Result<()> {
    get_app_handle()
        .shell()
        .command("explorer")
        .args(["/select,", &path])
        .spawn()?;
    Ok(())
}

#[tauri::command(async)]
fn open_file(path: String) -> Result<()> {
    get_app_handle()
        .shell()
        .command("explorer")
        .arg(&path)
        .spawn()?;
    Ok(())
}

#[tauri::command(async)]
async fn run_as_admin(program: String, args: Vec<String>) -> Result<()> {
    let command = if args.is_empty() {
        format!("Start-Process '{}' -Verb runAs", program)
    } else {
        format!(
            "Start-Process '{}' -Verb runAs -ArgumentList '{}'",
            program,
            args.join(" ")
        )
    };
    PwshScript::new(command).execute().await?;
    Ok(())
}

#[tauri::command(async)]
async fn run(program: PathBuf, args: Vec<String>, working_dir: Option<PathBuf>) -> Result<()> {
    // we create a link file to trick with explorer into a separated process
    // and without elevation in case Seelen UI was running as admin
    // this could take some delay like is creating a file but just are some milliseconds
    // and this exposed funtion is intended to just run certain times
    let lnk_file =
        WindowsApi::create_temp_shortcut(&program, &args.join(" "), working_dir.as_deref())?;
    get_app_handle()
        .shell()
        .command("explorer")
        .arg(&lnk_file)
        .status()
        .await?;
    std::fs::remove_file(&lnk_file)?;
    Ok(())
}

#[tauri::command(async)]
fn is_dev_mode() -> bool {
    tauri::is_dev()
}

#[tauri::command(async)]
fn is_appx_package() -> bool {
    is_running_as_appx()
}

#[tauri::command(async)]
pub fn get_user_envs() -> HashMap<String, String> {
    std::env::vars().collect::<HashMap<String, String>>()
}

// https://docs.rs/tauri/latest/tauri/window/struct.WindowBuilder.html#known-issues
// https://github.com/tauri-apps/wry/issues/583
#[tauri::command(async)]
fn show_app_settings() {
    log_error!(Seelen::show_settings());
}

#[tauri::command(async)]
async fn set_auto_start(enabled: bool) -> Result<()> {
    Seelen::set_auto_start(enabled)
}

#[tauri::command(async)]
async fn get_auto_start_status() -> Result<bool> {
    Seelen::is_auto_start_enabled()
}

#[tauri::command(async)]
fn switch_workspace(idx: usize) -> Result<()> {
    get_vd_manager().switch_to(idx)
}

#[tauri::command(async)]
fn send_keys(keys: String) -> Result<()> {
    Keyboard::new().send_keys(&keys)
}

#[tauri::command(async)]
fn get_icon(path: Option<PathBuf>, umid: Option<String>) -> Option<PathBuf> {
    let mut icon = None;
    if let Some(umid) = umid {
        icon = extract_and_save_icon_umid(&umid.into()).ok();
    }
    match path {
        Some(path) if icon.is_none() => extract_and_save_icon_from_file(&path).ok(),
        _ => icon,
    }
}

#[tauri::command(async)]
fn is_virtual_desktop_supported() -> bool {
    virtual_desktop_supported()
}

#[tauri::command(async)]
fn simulate_fullscreen(webview: WebviewWindow<tauri::Wry>, value: bool) -> Result<()> {
    let handle = HWND(webview.hwnd()?.0);
    let monitor = WindowsApi::monitor_from_window(handle);
    let event = if value {
        WinEvent::SyntheticFullscreenStart(SyntheticFullscreenData { handle, monitor })
    } else {
        WinEvent::SyntheticFullscreenEnd(SyntheticFullscreenData { handle, monitor })
    };
    HookManager::event_tx().send((event, Window::from(handle)))?;
    Ok(())
}

#[tauri::command(async)]
async fn check_for_updates() -> Result<bool> {
    Ok(utils::updater::check_for_updates().await?.is_some())
}

#[tauri::command(async)]
async fn get_foreground_window_color() -> Result<Color> {
    let window = Window::from(WindowsApi::get_foreground_window());
    if !window.is_visible()
        || !window.is_maximized()
        || window.is_minimized()
        || window.is_desktop()
    {
        return Ok(Color::default());
    }
    let hdc = DeviceContext::create(None);
    let rect = window.inner_rect()?;
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

pub fn register_invoke_handler(app_builder: Builder<Wry>) -> Builder<Wry> {
    use crate::seelen_rofi::handler::*;
    use crate::seelen_weg::handler::*;
    use crate::seelen_wm_v2::handler::*;
    use crate::state::infrastructure::*;
    use crate::system::brightness::*;

    use crate::modules::language::*;
    use crate::modules::media::infrastructure::*;
    use crate::modules::monitors::infrastructure::*;
    use crate::modules::network::infrastructure::*;
    use crate::modules::notifications::infrastructure::*;
    use crate::modules::power::infrastructure::*;
    use crate::modules::system_settings::infrastructure::*;
    use crate::modules::tray::infrastructure::*;
    use crate::modules::user::infrastructure::*;

    app_builder.invoke_handler(command_handler_list!())
}
