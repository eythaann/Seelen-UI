pub mod cli;

use tauri::WebviewWindow;
use windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE;

use crate::{error_handler::Result, seelen::get_app_handle, windows_api::WindowsApi};

pub struct SeelenRofi {
    window: WebviewWindow,
}

impl SeelenRofi {
    pub const TITLE: &str = "Seelen App Launcher";
    pub const TARGET: &str = "seelen-rofi";

    pub fn new() -> Result<Self> {
        Ok(Self {
            window: Self::create_window()?,
        })
    }

    pub fn show(&self) -> Result<()> {
        let rc_monitor = WindowsApi::monitor_info(WindowsApi::monitor_from_cursor_point())?
            .monitorInfo
            .rcMonitor;
        WindowsApi::move_window(self.window.hwnd()?, &rc_monitor)?;
        WindowsApi::set_position(self.window.hwnd()?, None, &rc_monitor, SWP_NOACTIVATE)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.window.show()?;
        Ok(())
    }

    pub fn hide(&self) -> Result<()> {
        self.window.hide()?;
        Ok(())
    }

    fn create_window() -> Result<WebviewWindow> {
        let window = tauri::WebviewWindowBuilder::new(
            &get_app_handle(),
            Self::TARGET,
            tauri::WebviewUrl::App("seelen_rofi/index.html".into()),
        )
        .title(Self::TITLE)
        .maximizable(false)
        .minimizable(false)
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
