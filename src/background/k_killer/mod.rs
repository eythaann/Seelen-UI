pub mod cli;
pub mod handler;
pub mod hook;

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use windows::Win32::Foundation::HWND;

use crate::{
    error_handler::Result,
    seelenweg::SeelenWeg,
    utils::virtual_desktop::VirtualDesktopManager,
    windows_api::WindowsApi
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
    pub current_virtual_desktop: String,
    paused: bool,
}

impl WindowManager {
    const TARGET: &'static str = "k_killer";

    pub fn new(handle: AppHandle<Wry>) -> Self {
        log::info!("Creating Tiling Windows Manager");
        let virtual_desktop = VirtualDesktopManager::get_current_virtual_desktop().expect("Failed to get current virtual desktop");
        Self {
            window: Self::create_window(&handle).expect("Failed to create Manager Container"),
            handle,
            hwnds: Vec::new(),
            current_virtual_desktop: virtual_desktop.id(),
            paused: true, // paused until complete_window_setup is called
        }
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
        .build()?;

        window.set_ignore_cursor_events(true)?;

        Ok(window)
    }

    fn complete_window_setup(&mut self) -> Result<()> {
        log::info!("Tiling Windows Manager Created");
        self.paused = false;
        self.handle
            .emit_to(Self::TARGET, "set-active-workspace", &self.current_virtual_desktop)?;
        Ok(())
    }

    pub fn contains(&self, hwnd: HWND) -> bool {
        self.hwnds.contains(&hwnd.0)
    }

    pub fn _hwnd(&self) -> HWND {
        HWND(self.window.hwnd().expect("can't get Self Window handle").0)
    }

    pub fn should_handle(hwnd: HWND) -> bool {
        SeelenWeg::should_handle_hwnd(hwnd) 
        && !WindowsApi::is_iconic(hwnd)
        // Without admin some apps does not return the exe path so this should be unmanaged
        && WindowsApi::exe_path(hwnd).is_ok()
    }

    pub fn set_active_window(&self, hwnd: HWND) -> Result<()> {
        if WindowsApi::get_window_text(hwnd) == "Task Switching" {
            return Ok(());
        }

        let hwnd = match self.contains(hwnd) {
            true => {
                self.psudo_resume()?;
                hwnd
            },
            false => {
                self.pseudo_pause()?;
                HWND(0) // avoid rerenders on multiple unmanaged focus
            },
        };
 
        self.handle
            .emit_to(Self::TARGET, "set-active-window", hwnd.0)?;
        Ok(())
    }

    pub fn set_active_workspace(&mut self, virtual_desktop_id: String) -> Result<()> {
        if virtual_desktop_id == self.current_virtual_desktop {
            return Ok(());
        }
        log::trace!("Setting active workspace to: {:?}", virtual_desktop_id);
        self.discard_reservation()?;
        self.current_virtual_desktop = virtual_desktop_id;
        self.handle
            .emit_to(Self::TARGET, "set-active-workspace", &self.current_virtual_desktop)?;
        Ok(())
    }

    pub fn add_hwnd(&mut self, hwnd: HWND) -> Result<bool> {
        if self.paused || self.contains(hwnd) {
            return Ok(false)
        }

        let mut desktop_to_add = self.current_virtual_desktop.clone();
        if WindowsApi::is_cloaked(hwnd)? {
            desktop_to_add = format!("{:?}", WindowsApi::get_virtual_desktop_id(hwnd)?);
        }

        log::trace!("Adding {} <=> {:?} to desktop: {}", hwnd.0, WindowsApi::get_window_text(hwnd), desktop_to_add);

        self.hwnds.push(hwnd.0);
        self.handle.emit_to(
            Self::TARGET,
            "add-window",
            AddWindowPayload {
                hwnd: hwnd.0,
                desktop_id: desktop_to_add,
            },
        )?;
        Ok(true)
    }

    pub fn remove_hwnd_no_emit(&mut self, hwnd: HWND) -> bool {
        if self.paused || !self.contains(hwnd) {
            return false
        }
        self.hwnds.retain(|&x| x != hwnd.0);
        true
    }

    pub fn remove_hwnd(&mut self, hwnd: HWND) -> Result<bool> {
        if !self.remove_hwnd_no_emit(hwnd) {
            return Ok(false)
        }
        self.handle.emit_to(Self::TARGET, "remove-window", hwnd.0)?;
        Ok(true)
    }

    pub fn force_retiling(&self) -> Result<()> {
        self.handle.emit_to(Self::TARGET, "force-retiling", ())?;
        Ok(())
    }

    pub fn pseudo_pause(&self) -> Result<()> {
        self.window.set_always_on_top(false)?;
        self.window.set_always_on_bottom(true)?;
        Ok(())
    }

    pub fn pause(&mut self, action: bool, visuals: bool) -> Result<()> {
        self.paused = action;
        if visuals {
            match action {
                true => self.pseudo_pause()?,
                false => self.psudo_resume()?,
            }
        }
        Ok(())
    }

    pub fn psudo_resume(&self) -> Result<()> {
        self.window.set_always_on_bottom(false)?;
        self.window.set_always_on_top(true)?;
        Ok(())
    }
}
