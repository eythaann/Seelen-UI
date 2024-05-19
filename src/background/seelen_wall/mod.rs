use tauri::{AppHandle, WebviewWindow, Wry};
use windows::Win32::{Graphics::Gdi::HMONITOR, UI::WindowsAndMessaging::{SWP_NOACTIVATE, SWP_NOSIZE}};

use crate::{error_handler::Result, seelen::get_app_handle, windows_api::WindowsApi};

pub struct SeelenWall {
    window: WebviewWindow,
}

impl SeelenWall {
    pub fn new(monitor: isize) -> Result<Self> {
        let manager = get_app_handle();
        Ok(Self {
            window: Self::create_window(&manager, monitor)?,
        })
    }

    fn create_window(manager: &AppHandle<Wry>, monitor: isize) -> Result<WebviewWindow> {
        let monitor_info = WindowsApi::monitor_info(HMONITOR(monitor))?;
        let rc_monitor = monitor_info.monitorInfo.rcMonitor;

        let window = tauri::WebviewWindowBuilder::new(
            manager,
            format!("seelen-wall/{}", monitor),
            tauri::WebviewUrl::App("the_wall/index.html".into()),
        )
        .title("Seelen Wall")
        .maximizable(false)
        .minimizable(false)
        .closable(false)
        .resizable(false)
        .visible(false)
        .decorations(false)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_bottom(true)
        .visible_on_all_workspaces(true)
        .disable_file_drop_handler()
        .build()?;

        window.set_ignore_cursor_events(true)?;

        // pre set position for resize in case of multiples dpi
        WindowsApi::set_position(window.hwnd()?, None, &rc_monitor, SWP_NOSIZE)?;
        WindowsApi::set_position(window.hwnd()?, None, &rc_monitor, SWP_NOACTIVATE)?;

        Ok(window)
    }
}
