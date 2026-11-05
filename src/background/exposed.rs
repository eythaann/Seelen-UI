use std::{collections::HashMap, path::PathBuf};

use seelen_core::system_state::RelaunchArguments;

use slu_ipc::{ServiceIpc, messages::SvcAction};

use crate::{
    app::{SeelenUI, get_app_handle},
    error::Result,
    platform::shell,
    utils::{
        self,
        icon_extractor::{request_icon_extraction_from_file, request_icon_extraction_from_umid},
    },
    widgets::{
        permissions::{WidgetPerm, request_widget_permission},
        popups::shortcut_registering::{abort_shortcut_registering, begin_shortcut_registering},
    },
};

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
        shell::open(&path)
    }

    pub fn select_file_on_explorer(path: PathBuf) -> Result<()> {
        shell::reveal(&path)
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
        shell::execute(&program, args.as_deref(), working_dir.as_deref(), elevated)
    }

    pub fn app_exit() {
        get_app_handle().exit(0);
    }

    pub fn app_restart() {
        get_app_handle().restart();
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

    /// Resolves when the user finishes: the shortcut, or `None` if cancelled, no keys were
    /// pressed or another registering is already in progress.
    pub async fn request_to_user_input_shortcut() -> Result<Option<Vec<String>>> {
        // claimed before starting: the service reports the first (empty) shortcut
        // before its call returns
        let Some(result) = begin_shortcut_registering() else {
            return Ok(None);
        };
        if let Err(err) = ServiceIpc::send(SvcAction::StartShortcutRegistration).await {
            abort_shortcut_registering();
            return Err(err.into());
        }
        // a dropped responder means it ended without a result
        Ok(result.await.unwrap_or(None))
    }
}
