use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;

use owo_colors::OwoColorize;
use seelen_core::constants::SUPPORTED_LANGUAGES;
use seelen_core::resource::ResourceText;
use seelen_core::state::RelaunchArguments;
use seelen_core::{command_handler_list, system_state::Color};

use tauri::{Builder, WebviewWindow, Wry};
use tauri_plugin_shell::ShellExt;
use translators::Translator;
use windows::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW};

use crate::app::{get_app_handle, Seelen};
use crate::error::Result;

use crate::utils;
use crate::utils::constants::SEELEN_COMMON;
use crate::utils::icon_extractor::{
    request_icon_extraction_from_file, request_icon_extraction_from_umid,
};
use crate::windows_api::hdc::DeviceContext;
use crate::windows_api::window::Window;
use crate::windows_api::WindowsApi;

#[tauri::command(async)]
pub fn open_file(path: String) -> Result<()> {
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
    program: String,
    args: Option<RelaunchArguments>,
    working_dir: Option<PathBuf>,
    elevated: bool,
) -> Result<()> {
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

pub async fn translate_file(path: PathBuf, source_lang: Option<String>) -> Result<()> {
    let file = std::fs::File::open(&path)?;
    let mut texts: ResourceText = serde_yaml::from_reader(file)?;

    let code = match source_lang {
        Some(source_lang) => source_lang,
        None => "en".to_string(),
    };

    if !texts.has(&code) {
        return Err(format!("Source Language ({code}) not found.").into());
    }

    let source = texts.get(&code).to_owned();
    let total = SUPPORTED_LANGUAGES.len();

    let longest_lang = SUPPORTED_LANGUAGES
        .iter()
        .map(|lang| lang.en_label.len())
        .max()
        .unwrap_or(0);

    for (idx, lang) in SUPPORTED_LANGUAGES.iter().enumerate() {
        let step = if idx < 9 {
            format!("0{}", idx + 1)
        } else {
            (idx + 1).to_string()
        };

        // fill with spaces to fit max length
        let label = format!(
            "{}{}",
            lang.en_label,
            " ".repeat(longest_lang - lang.en_label.len())
        );

        if texts.has(lang.value) {
            println!(
                "[{step}/{total}] {} => {}",
                label.bright_black(),
                "Skipped".bright_black()
            );
            continue;
        }

        match _translate_text(&source, &code, lang.value).await {
            Ok(value) => {
                println!(
                    "[{step}/{total}] {} => \"{}\"",
                    label.bold().bright_green(),
                    value
                );
                texts.set(lang.value.to_string(), value);
            }
            Err(err) => {
                eprintln!(
                    "[{step}/{total}] {} => Error translating to {} ({}): {}",
                    label.bold().bright_red(),
                    lang.en_label,
                    lang.value,
                    err
                );
            }
        }
    }

    let file = std::fs::File::create(&path)?;
    serde_yaml::to_writer(file, &texts)?;
    Ok(())
}

async fn _translate_text(source: &str, source_lang: &str, mut target_lang: &str) -> Result<String> {
    use translators::GoogleTranslator;
    let translator = GoogleTranslator::default();

    if target_lang == "zh" {
        target_lang = "zh-CN";
    }

    if target_lang == "pt" {
        target_lang = "pt-BR";
    }

    let translated = translator
        .translate_async(source, source_lang, target_lang)
        .await?;
    Ok(translated)
}

pub fn register_invoke_handler(app_builder: Builder<Wry>) -> Builder<Wry> {
    use crate::cli::*;
    use crate::state::infrastructure::*;
    use crate::virtual_desktops::handlers::*;
    use crate::widgets::popups::handlers::*;
    use crate::widgets::weg::handler::*;
    use crate::widgets::window_manager::handler::*;
    use crate::widgets::*;

    use crate::modules::apps::infrastructure::*;
    use crate::modules::media::devices::infrastructure::*;
    use crate::modules::media::players::infrastructure::*;
    use crate::modules::monitors::infrastructure::*;
    use crate::modules::network::infrastructure::*;
    use crate::modules::notifications::infrastructure::*;
    use crate::modules::power::infrastructure::*;
    use crate::modules::radios::bluetooth::handlers::*;
    use crate::modules::radios::handlers::*;
    use crate::modules::start::infrastructure::*;
    use crate::modules::system_settings::infrastructure::*;
    use crate::modules::system_settings::language::infrastructure::*;
    use crate::modules::system_tray::infrastructure::*;
    use crate::modules::user::infrastructure::*;

    use crate::resources::commands::*;

    app_builder.invoke_handler(command_handler_list!())
}
