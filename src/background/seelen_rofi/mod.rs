pub mod cli;
pub mod handler;

use std::{ffi::OsStr, path::PathBuf};

use serde::Serialize;
use tauri::{path::BaseDirectory, Manager, WebviewWindow};
use windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE;

use crate::{
    error_handler::Result, log_error, seelen::get_app_handle,
    seelen_weg::icon_extractor::extract_and_save_icon_v2, utils::constants::Icons,
    windows_api::WindowsApi,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeelenRofiApp {
    pub label: String,
    pub icon: PathBuf,
    pub path: PathBuf,
}

pub struct SeelenRofi {
    window: WebviewWindow,
    pub apps: Vec<SeelenRofiApp>,
}

impl Drop for SeelenRofi {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        log_error!(self.window.destroy());
    }
}

impl SeelenRofi {
    pub const TITLE: &str = "Seelen App Launcher";
    pub const TARGET: &str = "seelen-launcher";

    pub fn new() -> Result<Self> {
        log::info!("Creating {}", Self::TARGET);
        Ok(Self {
            // apps should be loaded first because it takes a long time on start and its needed by webview
            apps: Self::load_apps()?,
            window: Self::create_window()?,
        })
    }

    fn load_dir(dir: PathBuf) -> Result<Vec<SeelenRofiApp>> {
        let mut apps = Vec::new();
        for entry in std::fs::read_dir(dir)?.flatten() {
            let file_type = entry.file_type()?;
            let path = entry.path();

            if file_type.is_dir() {
                match Self::load_dir(path) {
                    Ok(app) => apps.extend(app),
                    Err(e) => log::error!("{:?}", e),
                }
                continue;
            }

            if file_type.is_file() && path.extension() != Some(OsStr::new("ini")) {
                apps.push(SeelenRofiApp {
                    label: path.file_stem().unwrap().to_string_lossy().to_string(),
                    icon: extract_and_save_icon_v2(&path).unwrap_or_else(|_| Icons::missing_app()),
                    path,
                })
            }
        }
        Ok(apps)
    }

    fn load_apps() -> Result<Vec<SeelenRofiApp>> {
        let mut result = Vec::new();

        let apps = Self::load_dir(PathBuf::from(
            r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs",
        ))?;
        result.extend(apps);

        let apps = Self::load_dir(get_app_handle().path().resolve(
            r"Microsoft\Windows\Start Menu\Programs",
            BaseDirectory::Data,
        )?)?;
        result.extend(apps);

        result.sort_by_key(|app| app.label.to_lowercase());
        Ok(result)
    }

    pub fn show(&self) -> Result<()> {
        let rc_monitor = WindowsApi::monitor_info(WindowsApi::monitor_from_cursor_point())?
            .monitorInfo
            .rcMonitor;
        WindowsApi::move_window(self.window.hwnd()?, &rc_monitor)?;
        WindowsApi::set_position(self.window.hwnd()?, None, &rc_monitor, SWP_NOACTIVATE)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.window.show()?;
        self.window.set_focus()?;
        Ok(())
    }

    pub fn hide(&self) -> Result<()> {
        self.window.hide()?;
        Ok(())
    }

    fn create_window() -> Result<WebviewWindow> {
        let window = tauri::WebviewWindowBuilder::new(
            get_app_handle(),
            Self::TARGET,
            tauri::WebviewUrl::App("seelen_rofi/index.html".into()),
        )
        .title(Self::TITLE)
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .resizable(false)
        .visible(false) // change to false after finish development
        .transparent(true)
        .shadow(false)
        .decorations(false)
        .skip_taskbar(true)
        .drag_and_drop(false)
        .always_on_top(true)
        .build()?;

        Ok(window)
    }
}
