pub mod launcher;
pub mod loader;
pub mod manager;
pub mod popups;
pub mod task_switcher;
pub mod toolbar;
pub mod wallpaper_manager;
pub mod webview;
pub mod weg;
pub mod window_manager;

use std::{path::PathBuf, sync::LazyLock};

use seelen_core::{
    handlers::SeelenEvent,
    state::{WidgetStatus, WidgetTriggerPayload},
};
use tauri::{Emitter, Manager};

use crate::{
    app::get_app_handle,
    error::Result,
    state::application::FULL_STATE,
    utils::{constants::SEELEN_COMMON, lock_free::SyncHashMap, WidgetWebviewLabel},
    widgets::manager::WIDGET_MANAGER,
};

static PENDING_TRIGGERS: LazyLock<SyncHashMap<WidgetWebviewLabel, WidgetTriggerPayload>> =
    LazyLock::new(SyncHashMap::new);

#[tauri::command(async)]
pub fn set_current_widget_status(
    webview: tauri::WebviewWindow,
    status: WidgetStatus,
) -> Result<()> {
    let label = WidgetWebviewLabel::try_from_raw(webview.label())?;
    WIDGET_MANAGER.set_status(&label, status);

    if let Some(pending) = PENDING_TRIGGERS.remove(&label) {
        get_app_handle().emit_to(label.raw, SeelenEvent::WidgetTriggered, &pending)?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn trigger_widget(payload: WidgetTriggerPayload) -> Result<()> {
    let state = FULL_STATE.load();
    if !state.is_widget_enabled(&payload.id) {
        return Err("Can't trigger a disabled widget".into());
    }

    let monitor_id = payload.monitor_id.as_ref().map(|id| id.to_string());
    let label = WidgetWebviewLabel::new(
        &payload.id,
        monitor_id.as_deref(),
        payload.instance_id.as_ref(),
    );

    if !WIDGET_MANAGER.is_ready(&label) {
        log::warn!("Trying to trigger widget that is not ready: {label}");
        PENDING_TRIGGERS.upsert(label.clone(), payload);

        WIDGET_MANAGER.groups.get(&label.widget_id, |c| {
            c.start_webview(&label);
        });
        return Ok(());
    }

    get_app_handle().emit_to(label.raw, SeelenEvent::WidgetTriggered, &payload)?;
    Ok(())
}

#[tauri::command(async)]
pub fn get_self_window_handle(webview: tauri::WebviewWindow<tauri::Wry>) -> Result<isize> {
    Ok(webview.hwnd()?.0 as isize)
}

pub fn show_settings() -> Result<()> {
    log::trace!("Show settings window");
    let label = WidgetWebviewLabel::new("@seelen/settings", None, None);
    let handle = get_app_handle();
    match handle.get_webview_window(&label.raw) {
        Some(window) => {
            window.unminimize()?;
            window.set_focus()?;
        }
        None => {
            let args = WebviewArgs::new().disable_gpu();
            tauri::WebviewWindowBuilder::new(
                handle,
                label.raw,
                tauri::WebviewUrl::App("react/settings/index.html".into()),
            )
            .title("Settings")
            .inner_size(800.0, 500.0)
            .min_inner_size(600.0, 400.0)
            .visible(false)
            .decorations(false)
            .center()
            .data_directory(args.data_directory())
            .additional_browser_args(&args.to_string())
            .build()?;
        }
    }
    Ok(())
}

pub struct WebviewArgs {
    pub args: Vec<String>,
}

impl WebviewArgs {
    const BASE_1: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";
    const BASE_2: &str = "--no-first-run --disable-site-isolation-trials --disable-background-timer-throttling --V8Maglev";

    pub fn new() -> Self {
        // --disk-cache-size=0
        Self {
            args: vec![Self::BASE_1.to_string(), Self::BASE_2.to_string()],
        }
    }

    pub fn with(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn disable_gpu(self) -> Self {
        self.with("--disable-gpu")
    }

    pub fn data_directory(&self) -> PathBuf {
        // remove bases
        let mut args = self.args.clone();
        args.remove(0);
        args.remove(0);

        if args.is_empty() {
            SEELEN_COMMON.app_cache_dir().to_path_buf()
        } else {
            SEELEN_COMMON
                .app_cache_dir()
                .join(args.join("").replace("-", ""))
        }
    }
}

impl std::fmt::Display for WebviewArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.args.join(" "))
    }
}
