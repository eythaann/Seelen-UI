pub mod cli;
pub mod handler;
pub mod hook;

use std::sync::atomic::{AtomicIsize, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    UI::WindowsAndMessaging::{EnumWindows, SWP_NOACTIVATE, WS_CAPTION, WS_EX_TOPMOST},
};

use crate::{
    apps_config::{AppExtraFlag, SETTINGS_BY_APP},
    error_handler::Result,
    seelen_weg::SeelenWeg,
    utils::virtual_desktop::VirtualDesktopManager,
    windows_api::WindowsApi,
};

#[derive(Serialize, Clone)]
struct AddWindowPayload {
    hwnd: isize,
    desktop_id: String,
    as_floating: bool,
}

pub struct WindowManager {
    handle: AppHandle<Wry>,
    window: WebviewWindow,
    tiled_handles: Vec<isize>,
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
        let virtual_desktop = VirtualDesktopManager::get_current_virtual_desktop()
            .expect("Failed to get current virtual desktop");
        Self {
            window: Self::create_window(&handle).expect("Failed to create Manager Container"),
            handle,
            tiled_handles: Vec::new(),
            floating_handles: Vec::new(),
            current_virtual_desktop: virtual_desktop.id(),
            paused: true, // paused until complete_window_setup is called
        }
    }

    pub fn complete_window_setup(&mut self) -> Result<()> {
        log::info!("Tiling Windows Manager Created");
        self.paused = false;
        self.handle.emit_to(
            Self::TARGET,
            "set-active-workspace",
            &self.current_virtual_desktop,
        )?;
        Self::enum_windows();
        Ok(())
    }

    pub fn is_managed(&self, hwnd: HWND) -> bool {
        self.tiled_handles.contains(&hwnd.0) || self.floating_handles.contains(&hwnd.0)
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
        let v_desktop = VirtualDesktopManager::get_window_virtual_desktop(hwnd)
            .or_else(|_| VirtualDesktopManager::get_current_virtual_desktop())?;
        if v_desktop.id() != self.current_virtual_desktop {
            self.set_active_workspace(v_desktop.id())?;
        }
        log::trace!(
            "Setting active window to {} <=> {:?} on {}",
            hwnd.0,
            WindowsApi::get_window_text(hwnd),
            v_desktop.id()[0..8].to_string()
        );
        let hwnd = match self.is_managed(hwnd)
            && !self.is_floating(hwnd)
            && !WindowsApi::is_maximized(hwnd)
        {
            true => {
                self.pseudo_resume()?;
                hwnd
            }
            false => {
                self.pseudo_pause()?;
                HWND(0)
            }
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
        self.handle.emit_to(
            Self::TARGET,
            "set-active-workspace",
            &self.current_virtual_desktop,
        )?;
        Ok(())
    }

    pub fn add_hwnd(&mut self, hwnd: HWND) -> Result<bool> {
        if self.paused || self.is_managed(hwnd) {
            return Ok(false);
        }

        let mut desktop_to_add = self.current_virtual_desktop.clone();
        if WindowsApi::is_cloaked(hwnd)? {
            desktop_to_add = format!("{:?}", WindowsApi::get_virtual_desktop_id(hwnd)?);
        }

        log::trace!(
            "Adding {}({}) <=> {:?} on desktop: {}",
            WindowsApi::exe(hwnd).unwrap_or_default(),
            hwnd.0,
            WindowsApi::get_window_text(hwnd),
            desktop_to_add
        );

        let mut as_floating = false;
        if let Some(config) = SETTINGS_BY_APP.lock().get_by_window(hwnd) {
            as_floating = config.options_contains(AppExtraFlag::Float);
        }

        if as_floating {
            self.floating_handles.push(hwnd.0);
        } else {
            self.tiled_handles.push(hwnd.0);
        }

        self.handle.emit_to(
            Self::TARGET,
            "add-window",
            AddWindowPayload {
                hwnd: hwnd.0,
                desktop_id: desktop_to_add,
                as_floating,
            },
        )?;
        Ok(true)
    }

    pub fn emit_send_to_workspace(&mut self, hwnd: HWND, desktop_id: String) -> Result<()> {
        let mut as_floating = false;
        if let Some(config) = SETTINGS_BY_APP.lock().get_by_window(hwnd) {
            as_floating = config.options_contains(AppExtraFlag::Float);
        }
        self.handle.emit_to(
            Self::TARGET,
            "move-window-to-workspace",
            AddWindowPayload {
                hwnd: hwnd.0,
                desktop_id,
                as_floating,
            },
        )?;
        Ok(())
    }

    pub fn remove_hwnd_no_emit(&mut self, hwnd: HWND) -> bool {
        if self.paused || !self.is_managed(hwnd) {
            return false;
        }
        self.tiled_handles.retain(|&x| x != hwnd.0);
        true
    }

    /** triggered when a window is bounced by the front-end on adding action */
    pub fn bounce_handle(&mut self, hwnd: HWND) {
        if self.remove_hwnd_no_emit(hwnd) {
            self.floating_handles.push(hwnd.0);
        }
    }

    pub fn remove_hwnd(&mut self, hwnd: HWND) -> Result<bool> {
        if !self.remove_hwnd_no_emit(hwnd) {
            return Ok(false);
        }
        log::trace!(
            "Removing {} <=> {:?}",
            hwnd.0,
            WindowsApi::get_window_text(hwnd)
        );
        self.handle.emit_to(Self::TARGET, "remove-window", hwnd.0)?;
        Ok(true)
    }

    pub fn force_retiling(&self) -> Result<()> {
        log::trace!("Forcing retiling");
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
}

// UTILS AND STATICS
impl WindowManager {
    pub fn should_manage(hwnd: HWND) -> bool {
        let mut settings_by_app = SETTINGS_BY_APP.lock();
        if let Some(config) = settings_by_app.get_by_window(hwnd) {
            return config.options_contains(AppExtraFlag::Force) || {
                !config.options_contains(AppExtraFlag::Unmanage)
                    && !config.options_contains(AppExtraFlag::Pinned)
                    && Self::is_manageable_window(hwnd, false)
            };
        }
        Self::is_manageable_window(hwnd, false)
    }

    pub fn is_manageable_window(hwnd: HWND, ignore_cloaked: bool) -> bool {
        let exe = WindowsApi::exe(hwnd);

        if let Ok(exe) = &exe {
            if exe.ends_with("ApplicationFrameHost.exe") && SeelenWeg::is_real_window(hwnd, true) {
                return true;
            }
        }

        // Without admin some apps does not return the exe path so these should be unmanaged
        exe.is_ok()
        && SeelenWeg::is_real_window(hwnd, true)
        // Ignore windows without a title bar, and top most windows normally are widgets or tools so they should not be managed
        && (WindowsApi::get_styles(hwnd).contains(WS_CAPTION) && !WindowsApi::get_ex_styles(hwnd).contains(WS_EX_TOPMOST))
        && !WindowsApi::is_iconic(hwnd)
        && (ignore_cloaked || !WindowsApi::is_cloaked(hwnd).unwrap_or(false))
    }

    fn create_window(handle: &AppHandle<Wry>) -> Result<WebviewWindow> {
        let window = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            handle,
            Self::TARGET,
            tauri::WebviewUrl::App("seelen_wm/index.html".into()),
        )
        .title("Seelen Window Manager")
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

        let monitor_info = WindowsApi::monitor_info(WindowsApi::primary_monitor())?;
        let work_area = monitor_info.monitorInfo.rcWork;
        WindowsApi::set_position(window.hwnd()?, None, &work_area, SWP_NOACTIVATE)?;

        Ok(window)
    }

    const NEXT: AtomicIsize = AtomicIsize::new(0);
    unsafe extern "system" fn get_next_by_order_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if WindowManager::is_manageable_window(hwnd, false) && hwnd.0 != lparam.0 {
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
