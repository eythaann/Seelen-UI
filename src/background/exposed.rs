use std::{collections::HashMap, os::windows::process::CommandExt, path::PathBuf};

use seelen_core::{
    command_handler_list,
    system_state::{AppBarEdge, Color, RelaunchArguments, StartMenuLayout, StartMenuLayoutItem},
    Rect,
};

use tauri::{Builder, WebviewWindow, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::{
    Foundation::{HWND, RECT},
    System::Threading::{CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW},
};

use crate::{
    app::{get_app_handle, Seelen},
    error::Result,
    utils::{
        self,
        constants::SEELEN_COMMON,
        icon_extractor::{request_icon_extraction_from_file, request_icon_extraction_from_umid},
        pwsh::PwshScript,
    },
    widgets::permissions::{request_widget_permission, WidgetPerm},
    windows_api::{
        hdc::DeviceContext, string_utils::WindowsString, window::Window, AppBarData, WindowsApi,
    },
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
    Seelen::set_auto_start(enabled)
}

#[tauri::command(async)]
async fn get_auto_start_status() -> Result<bool> {
    Seelen::is_auto_start_enabled()
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
fn register_app_bar(webview: tauri::WebviewWindow, rect: Rect, edge: AppBarEdge) -> Result<()> {
    let hwnd = HWND(webview.hwnd()?.0);
    let rect = RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };

    let mut app_bar = AppBarData::from_handle(hwnd);
    app_bar.set_rect(rect);
    app_bar.set_edge(edge);
    app_bar.register_as_new_bar();
    Ok(())
}

#[tauri::command(async)]
fn unregister_app_bar(webview: tauri::WebviewWindow) -> Result<()> {
    let hwnd = HWND(webview.hwnd()?.0);
    let mut app_bar = AppBarData::from_handle(hwnd);
    app_bar.unregister_bar();
    Ok(())
}

pub fn register_invoke_handler(app_builder: Builder<Wry>) -> Builder<Wry> {
    use crate::cli::*;
    use crate::state::infrastructure::*;
    use crate::virtual_desktops::handlers::*;
    use crate::widgets::permissions::*;
    use crate::widgets::popups::handlers::*;
    use crate::widgets::weg::handler::*;
    use crate::widgets::window_manager::handler::*;
    use crate::widgets::*;

    use crate::backups::infrastructure::*;
    use crate::modules::apps::infrastructure::*;
    use crate::modules::clipboard::infrastructure::*;
    use crate::modules::focus_assist::infrastructure::*;
    use crate::modules::fonts::infrastructure::*;
    use crate::modules::media::devices::infrastructure::*;
    use crate::modules::media::players::infrastructure::*;
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
    use crate::session::infrastructure::*;

    use crate::resources::commands::*;

    app_builder.invoke_handler(command_handler_list!())
}
