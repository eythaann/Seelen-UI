use std::{collections::HashMap, os::windows::process::CommandExt, path::PathBuf};

use seelen_core::system_state::{RelaunchArguments, StartMenuLayout, StartMenuLayoutItem};

use slu_ipc::{ServiceIpc, messages::SvcAction};
use tauri::WebviewWindow;
use tauri_plugin_shell::ShellExt;
use windows::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW};

use crate::{
    app::{SeelenUI, get_app_handle},
    error::Result,
    utils::{
        self,
        constants::SEELEN_COMMON,
        icon_extractor::{request_icon_extraction_from_file, request_icon_extraction_from_umid},
        pwsh::PwshScript,
    },
    widgets::{
        permissions::{WidgetPerm, request_widget_permission},
        popups::shortcut_registering::REG_SHORTCUT_DATA,
    },
    windows_api::{WindowsApi, string_utils::WindowsString},
};

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

impl crate::tauri_handlers::Handlers {
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

    pub fn open_file(webview: tauri::WebviewWindow, path: String) -> Result<()> {
        request_widget_permission(&webview, WidgetPerm::OpenFile)?;
        open_file_inner(path)
    }

    pub fn select_file_on_explorer(path: PathBuf) -> Result<()> {
        let path = path.to_string_lossy().to_string();
        get_app_handle()
            .shell()
            .command(SEELEN_COMMON.system_dir().join("explorer.exe"))
            .args(["/select,", &path])
            .spawn()?;
        Ok(())
    }

    pub fn run(
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

    pub fn is_dev_mode() -> bool {
        tauri::is_dev()
    }

    pub fn has_fixed_runtime() -> bool {
        crate::utils::has_fixed_runtime()
    }

    pub fn is_appx_package() -> bool {
        crate::utils::is_running_as_appx()
    }

    pub fn get_user_envs() -> HashMap<String, String> {
        std::env::vars().collect::<HashMap<String, String>>()
    }

    pub fn set_auto_start(enabled: bool) -> Result<()> {
        SeelenUI::set_auto_start(enabled)
    }

    pub fn get_auto_start_status() -> Result<bool> {
        SeelenUI::is_auto_start_enabled()
    }

    // used to request icon extraction
    pub fn get_icon(path: Option<PathBuf>, umid: Option<String>) -> Result<()> {
        if let Some(umid) = umid {
            request_icon_extraction_from_umid(&umid.into());
        }
        if let Some(path) = path {
            request_icon_extraction_from_file(&path);
        }
        Ok(())
    }

    pub async fn check_for_updates() -> Result<bool> {
        Ok(utils::updater::check_for_updates().await?.is_some())
    }

    pub async fn install_last_available_update() -> Result<()> {
        let update = utils::updater::check_for_updates()
            .await?
            .ok_or("There is no update available")?;
        utils::updater::trace_update_intallation(update).await?;
        get_app_handle().restart();
        #[allow(unreachable_code)]
        Ok(())
    }

    pub async fn get_native_start_menu() -> Result<StartMenuLayout> {
        let output_path = SEELEN_COMMON.app_cache_dir().join("start-layout.json");
        let output_path_str = output_path.to_string_lossy().to_string();

        let script = PwshScript::new(format!("Export-StartLayout -Path '{}'", output_path_str))
            .inline_command();
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

    pub async fn request_to_user_input_shortcut(
        window: WebviewWindow,
        callback_event: String,
    ) -> Result<()> {
        ServiceIpc::send(SvcAction::StartShortcutRegistration).await?;

        let mut data = REG_SHORTCUT_DATA.lock();
        data.response_view_label = Some(window.label().to_string());
        data.response_event = Some(callback_event);
        Ok(())
    }
}
