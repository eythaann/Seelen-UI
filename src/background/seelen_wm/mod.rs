pub mod cli;
pub mod handler;

use std::{sync::atomic::{AtomicIsize, Ordering}, thread::sleep, time::Duration};

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use windows::Win32::{Foundation::{BOOL, HWND, LPARAM}, UI::WindowsAndMessaging::EnumWindows};

use crate::{
    error_handler::Result, seelen::SEELEN, seelen_weg::SeelenWeg, utils::virtual_desktop::VirtualDesktopManager, windows_api::WindowsApi
};

#[derive(Serialize, Clone)]
struct AddWindowPayload {
    hwnd: isize,
    desktop_id: String,
}

pub struct WindowManager {
    handle: AppHandle<Wry>,
    window: WebviewWindow,
    managed_handles: Vec<isize>,
    floating_handles: Vec<isize>,
    pub current_virtual_desktop: String,
    paused: bool,
}

impl WindowManager {
    pub const TARGET: &'static str = "seelen_wm";
    pub const VIRTUAL_PREVIEWS: [&'static str; 2] = [
        "Virtual desktop switching preview",
        "Virtual desktop hotkey switching preview",
    ];

    pub fn new(handle: AppHandle<Wry>) -> Self {
        log::info!("Creating Tiling Windows Manager");
        let virtual_desktop = VirtualDesktopManager::get_current_virtual_desktop().expect("Failed to get current virtual desktop");
        Self {
            window: Self::create_window(&handle).expect("Failed to create Manager Container"),
            handle,
            managed_handles: Vec::new(),
            floating_handles: Vec::new(),
            current_virtual_desktop: virtual_desktop.id(),
            paused: true, // paused until complete_window_setup is called
        }
    }

    pub fn complete_window_setup(&mut self) -> Result<()> {
        log::info!("Tiling Windows Manager Created");
        self.paused = false;
        self.handle
            .emit_to(Self::TARGET, "set-active-workspace", &self.current_virtual_desktop)?;
        Ok(())
    }

    pub fn is_managed(&self, hwnd: HWND) -> bool {
        self.managed_handles.contains(&hwnd.0) || self.floating_handles.contains(&hwnd.0)
    }

    pub fn is_floating(&self, hwnd: HWND) -> bool {
        self.floating_handles.contains(&hwnd.0)
    }

    pub fn _hwnd(&self) -> HWND {
        HWND(self.window.hwnd().expect("can't get Self Window handle").0)
    }

    pub fn set_active_window(&mut self, hwnd: HWND) -> Result<()> {
        if WindowsApi::get_window_text(hwnd) == "Task Switching" {
            return Ok(());
        }
        let vdesktop = VirtualDesktopManager::get_window_virtual_desktop(hwnd).or_else(|_| VirtualDesktopManager::get_current_virtual_desktop())?;
        if vdesktop.id() != self.current_virtual_desktop {
            self.set_active_workspace(vdesktop.id())?;
        }
        log::trace!("Setting active window to {} on {}", hwnd.0, vdesktop.id()[0..8].to_string());
        match self.is_managed(hwnd) {
            true => self.pseudo_resume()?,
            false => self.pseudo_pause()?,
        };
        self.handle
            .emit_to(Self::TARGET, "set-active-window", hwnd.0)?;
        Ok(())
    }

    pub fn set_active_workspace(&mut self, virtual_desktop_id: String) -> Result<()> {
        if virtual_desktop_id == self.current_virtual_desktop {
            return Ok(());
        }
        log::trace!("Setting active workspace to: {}", virtual_desktop_id);
        self.current_virtual_desktop = virtual_desktop_id;
        self.handle
            .emit_to(Self::TARGET, "set-active-workspace", &self.current_virtual_desktop)?;
        Ok(())
    }

    pub fn add_hwnd(&mut self, hwnd: HWND) -> Result<bool> {
        if self.paused || self.is_managed(hwnd) {
            return Ok(false)
        }

        let mut desktop_to_add = self.current_virtual_desktop.clone();
        if WindowsApi::is_cloaked(hwnd)? {
            desktop_to_add = format!("{:?}", WindowsApi::get_virtual_desktop_id(hwnd)?);
        }

        log::trace!("Adding {} <=> {:?} to desktop: {}", hwnd.0, WindowsApi::get_window_text(hwnd), desktop_to_add);

        self.managed_handles.push(hwnd.0);
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

    pub fn emit_send_to_workspace(&mut self, hwnd: HWND, desktop_id: String) -> Result<()>{
        self.handle.emit_to(
            Self::TARGET,
            "move-window-to-workspace",
            AddWindowPayload {
                hwnd: hwnd.0,
                desktop_id,
            },
        )?;
        Ok(())
    }

    pub fn remove_hwnd_no_emit(&mut self, hwnd: HWND) -> bool {
        if self.paused || !self.is_managed(hwnd) {
            return false
        }
        self.managed_handles.retain(|&x| x != hwnd.0);
        true
    }

    /** trigered when a window is bounced by the front-end on adding action */
    pub fn bounce_handle(&mut self, hwnd: HWND) {
        if self.remove_hwnd_no_emit(hwnd) {
            self.floating_handles.push(hwnd.0);
        }
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
                false => self.pseudo_resume()?,
            }
        }
        Ok(())
    }

    pub fn pseudo_resume(&self) -> Result<()> {
        self.window.set_always_on_bottom(false)?;
        self.window.set_always_on_top(true)?;
        Ok(())
    }

    pub fn force_focus(&mut self, hwnd: HWND) -> Result<()> {
        self.pause(true, false)?;
        WindowsApi::force_set_foregorund(hwnd)?;
        std::thread::spawn(|| -> Result<()> {
            sleep(Duration::from_millis(35));
            let mut seelen = SEELEN.lock();
            if let Some(wm) = seelen.wm_mut() {
                wm.pause(false, false)?;
            }
            Ok(())
        });

        Ok(())
    }
}

// UTILS AND STATICS
impl WindowManager {
    pub fn should_manage(hwnd: HWND) -> bool {
        Self::is_manageble_window(hwnd, false)
    }

    pub fn is_manageble_window(hwnd: HWND, ignore_cloaked: bool) -> bool {
        SeelenWeg::is_real_window(hwnd) 
        && !WindowsApi::is_iconic(hwnd)
        && (ignore_cloaked || !WindowsApi::is_cloaked(hwnd).unwrap_or(false))
        // Without admin some apps does not return the exe path so these should be unmanaged
        && WindowsApi::exe_path(hwnd).is_ok()
    }

    fn create_window(handle: &AppHandle<Wry>) -> Result<WebviewWindow> {
        let window = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            handle,
            Self::TARGET,
            tauri::WebviewUrl::App("seelen_wm/index.html".into()),
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

    const NEXT: AtomicIsize = AtomicIsize::new(0);
    unsafe extern "system" fn get_next_by_order_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if WindowManager::is_manageble_window(hwnd, false) && hwnd.0 != lparam.0 {
            Self::NEXT.store(hwnd.0, Ordering::SeqCst);
            return false.into();
        }
        true.into()
    }

    pub fn get_next_by_order(hwnd: HWND) -> Option<HWND> {
        Self::NEXT.store(0, Ordering::SeqCst);
        unsafe { EnumWindows(Some(Self::get_next_by_order_proc), LPARAM(hwnd.0)) }.ok();
        let result = Self::NEXT.load(Ordering::SeqCst);
        if result == 0 {
            return None;
        }
        Some(HWND(result))
    }
}