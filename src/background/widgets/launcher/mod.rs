pub mod cli;
pub mod handler;

use base64::Engine;
use seelen_core::system_state::StartMenuItem;
use tauri::WebviewWindow;
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::SWP_ASYNCWINDOWPOS};

use crate::{
    app::get_app_handle, error_handler::Result, log_error,
    modules::start::application::START_MENU_MANAGER, windows_api::WindowsApi,
};

pub struct SeelenRofi {
    window: WebviewWindow,
    pub apps: Vec<StartMenuItem>,
}

impl Drop for SeelenRofi {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        log_error!(self.window.destroy());
    }
}

impl SeelenRofi {
    pub const TITLE: &str = ".Seelen App Launcher";
    pub const TARGET: &str = "@seelen/launcher";

    pub fn new() -> Result<Self> {
        log::info!("Creating {}", Self::TARGET);
        Ok(Self {
            // apps should be loaded first because it takes a long time on start and its needed by webview
            apps: START_MENU_MANAGER.load().list.clone(),
            window: Self::create_window()?,
        })
    }

    pub fn hwnd(&self) -> Result<HWND> {
        Ok(HWND(self.window.hwnd()?.0))
    }

    pub fn show(&self) -> Result<()> {
        let rc_monitor = WindowsApi::monitor_info(WindowsApi::monitor_from_cursor_point())?
            .monitorInfo
            .rcMonitor;
        WindowsApi::move_window(self.hwnd()?, &rc_monitor)?;
        WindowsApi::set_position(self.hwnd()?, None, &rc_monitor, SWP_ASYNCWINDOWPOS)?;
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
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Self::TARGET);
        let window = tauri::WebviewWindowBuilder::new(
            get_app_handle(),
            label,
            tauri::WebviewUrl::App("launcher/index.html".into()),
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
