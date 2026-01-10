pub mod cli;
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
    Rect,
};
use tauri::{Emitter, Manager};
use windows::Win32::UI::WindowsAndMessaging::SWP_ASYNCWINDOWPOS;

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    state::application::FULL_STATE,
    utils::{constants::SEELEN_COMMON, lock_free::SyncHashMap, WidgetWebviewLabel},
    widgets::manager::WIDGET_MANAGER,
    windows_api::{input::Keyboard, WindowsApi},
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

#[tauri::command(async)]
pub fn set_self_position(webview: tauri::WebviewWindow<tauri::Wry>, rect: Rect) -> Result<()> {
    use windows::Win32::Foundation::{HWND, RECT};
    let hwnd = HWND(webview.hwnd()?.0);
    let rect = RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };
    // pre set position for resize in case of multiples dpi
    WindowsApi::move_window(hwnd, &rect)?;
    WindowsApi::set_position(hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;
    Ok(())
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

#[tauri::command(async)]
pub fn show_start_menu() -> Result<()> {
    let guard = FULL_STATE.load();
    if guard.is_widget_enabled(&"@seelen/apps-menu".into()) {
        trigger_widget(WidgetTriggerPayload::new("@seelen/apps-menu".into()))?;
        return Ok(());
    }
    // trick for showing the native start menu
    Keyboard::new().send_keys("{win}")
}

// https://docs.rs/tauri/latest/tauri/window/struct.WindowBuilder.html#known-issues
// https://github.com/tauri-apps/wry/issues/583
#[tauri::command(async)]
pub fn show_app_settings() {
    show_settings().log_error();
}

pub struct WebviewArgs {
    common_args: Vec<String>,
    extra_args: Vec<String>,
}

impl WebviewArgs {
    const BASE_OPT: &str = "--disable-features=translate,msWebOOUI,msPdfOOUI,msSmartScreenProtection,RendererAppContainer";
    const BASE_OPT2: &str =
        "--no-first-run --disable-site-isolation-trials --disable-background-timer-throttling";
    const PERFORMANCE_OPT: &str = "--enable-low-end-device-mode --in-process-gpu --V8Maglev";

    pub fn new() -> Self {
        Self {
            common_args: vec![
                Self::BASE_OPT.to_string(),
                Self::BASE_OPT2.to_string(),
                Self::PERFORMANCE_OPT.to_string(),
            ],
            extra_args: vec![],
        }
    }

    pub fn disable_gpu(self) -> Self {
        // if window manager is enabled (that is expected thing) having 2 processes one with gpu and another without,
        // is worse than having them together with gpu enabled so this is the reason why this is currently ignored.
        // self.extra_args.push("--disable-gpu --disable-software-rasterizer".to_string());
        self
    }

    pub fn data_directory(&self) -> PathBuf {
        if self.extra_args.is_empty() {
            SEELEN_COMMON.app_cache_dir().to_path_buf()
        } else {
            SEELEN_COMMON
                .app_cache_dir()
                .join(self.extra_args.join("").replace("-", ""))
        }
    }
}

impl std::fmt::Display for WebviewArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.common_args.join(" "))
    }
}
