pub mod handler;

use std::{thread::sleep, time::Duration};

use color_eyre::eyre::eyre;
use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use windows::Win32::Foundation::HWND;

use crate::{
    error_handler::Result, seelenweg::SeelenWeg, utils::compress_u128, windows_api::WindowsApi,
};

#[derive(Serialize, Clone)]
struct AddWindowPayload {
    hwnd: isize,
    desktop_id: String,
}

/** @Alias - K_Killer */
pub struct WindowManager {
    handle: AppHandle<Wry>,
    window: WebviewWindow,
    hwnds: Vec<isize>,
    current_virtual_desktop: String,
}

impl WindowManager {
    const TARGET: &'static str = "k_killer";

    pub fn new(handle: AppHandle<Wry>) -> Self {
        Self {
            window: Self::create_window(&handle).expect("Failed to create Manager Container"),
            handle,
            hwnds: Vec::new(),
            current_virtual_desktop: Default::default(),
        }
    }

    pub fn contains(&self, hwnd: HWND) -> bool {
        self.hwnds.contains(&hwnd.0)
    }

    fn create_window(handle: &AppHandle<Wry>) -> Result<WebviewWindow> {
        let window = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            handle,
            Self::TARGET,
            tauri::WebviewUrl::App("k_killer/index.html".into()),
        )
        .title("Seelen Window Manager")
        .position(0.0, 0.0)
        .maximizable(false)
        .minimizable(false)
        .resizable(false)
        .visible(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_bottom(true)
        /* .always_on_top(true) */
        .build()?;

        /* window.set_ignore_cursor_events(true)?; */

        Ok(window)
    }

    pub fn should_handle(hwnd: HWND) -> bool {
        SeelenWeg::should_handle_hwnd(hwnd) && !WindowsApi::is_iconic(hwnd)
    }

    pub fn add_hwnd(&mut self, hwnd: HWND) -> Result<()> {
        if self.contains(hwnd) {
            return Ok(());
        }

        let manager = WindowsApi::get_virtual_desktop_manager()?;
        unsafe {
            let mut desktop_id = manager.GetWindowDesktopId(hwnd)?.to_u128();
            let mut attempt = 0;
            while desktop_id == 0 && attempt < 10 {
                sleep(Duration::from_millis(30));
                desktop_id = manager.GetWindowDesktopId(hwnd)?.to_u128();
                attempt += 1;
            }
            

            if attempt == 10 && desktop_id == 0 {
                return Err(eyre!("Failed to get desktop id for: {hwnd:?}").into());
            }

            self.current_virtual_desktop = compress_u128(desktop_id);
            self.set_active_workspace(&self.current_virtual_desktop)?;
        }

        self.hwnds.push(hwnd.0);
        self.handle.emit_to(
            Self::TARGET,
            "add-window",
            AddWindowPayload {
                hwnd: hwnd.0,
                desktop_id: self.current_virtual_desktop.clone(),
            },
        )?;
        Ok(())
    }

    pub fn remove_hwnd(&mut self, hwnd: HWND) -> Result<()> {
        self.hwnds.retain(|&x| x != hwnd.0);
        self.handle.emit_to(Self::TARGET, "remove-window", hwnd.0)?;
        Ok(())
    }

    pub fn force_retiling(&self) -> Result<()> {
        self.handle.emit_to(Self::TARGET, "force-retiling", ())?;
        Ok(())
    }

    pub fn set_active_workspace(&self, id: &str) -> Result<()> {
        self.handle
            .emit_to(Self::TARGET, "set-active-workspace", id)?;
        Ok(())
    }
}
