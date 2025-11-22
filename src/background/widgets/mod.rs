pub mod launcher;
pub mod loader;
pub mod popups;
pub mod task_switcher;
pub mod toolbar;
pub mod wallpaper_manager;
pub mod weg;
pub mod window_manager;

use std::path::PathBuf;

use seelen_core::{handlers::SeelenEvent, state::WidgetTriggerPayload};
use tauri::{Emitter, Manager};

use crate::{
    app::get_app_handle,
    error::Result,
    utils::{constants::SEELEN_COMMON, WidgetWebviewLabel},
};

#[tauri::command(async)]
pub fn trigger_widget(payload: WidgetTriggerPayload) -> Result<()> {
    get_app_handle().emit(SeelenEvent::WidgetTriggered, &payload)?;
    Ok(())
}

#[tauri::command(async)]
pub fn get_self_window_handle(webview: tauri::WebviewWindow<tauri::Wry>) -> Result<isize> {
    Ok(webview.hwnd()?.0 as isize)
}

pub trait TrustedWidget {
    const ID: &'static str;

    fn title() -> String {
        Self::ID.to_string()
    }
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
