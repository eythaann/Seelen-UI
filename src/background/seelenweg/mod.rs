use tauri::{AppHandle, WebviewWindow, Wry};
use windows::{
    core::PCWSTR,
    Win32::UI::WindowsAndMessaging::{FindWindowW, ShowWindow, SW_HIDE, SW_SHOW},
};

use crate::error_handler::Result;

pub struct SeelenWeg {
    handle: AppHandle<Wry>,
}

impl SeelenWeg {
    pub fn new(handle: AppHandle<Wry>) -> Self {
        Self { handle }
    }

    pub fn create_window(app: &AppHandle) -> Result<WebviewWindow> {
        let window = tauri::WebviewWindowBuilder::new(
            app,
            "seelenpad",
            tauri::WebviewUrl::App("seelenpad/index.html".into()),
        )
        .inner_size(300.0, 300.0)
        .maximizable(false)
        .minimizable(false)
        .resizable(false)
        .title("Seelenpad")
        .visible(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()?;

        Ok(window)
    }

    pub fn start(&self) -> Result<()> {
        unsafe {
            let name: Vec<u16> = format!("Shell_TrayWnd\0").encode_utf16().collect();
            let taskbar = FindWindowW(PCWSTR(name.as_ptr()), PCWSTR::null());
            ShowWindow(taskbar, SW_HIDE);
        }
        Self::create_window(&self.handle)?;
        Ok(())
    }

    pub fn stop(&self) {
        unsafe {
            let name: Vec<u16> = format!("Shell_TrayWnd\0").encode_utf16().collect();
            let taskbar = FindWindowW(PCWSTR(name.as_ptr()), PCWSTR::null());
            ShowWindow(taskbar, SW_SHOW);
        }
    }
}
