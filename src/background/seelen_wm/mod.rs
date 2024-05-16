pub mod cli;
pub mod handler;
pub mod hook;

use std::sync::atomic::{AtomicIsize, Ordering};

use getset::{Getters, MutGetters};
use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{
        EnumWindows, HWND_BOTTOM, HWND_TOPMOST, SWP_NOACTIVATE, WS_CAPTION, WS_EX_TOPMOST,
    },
};

use crate::{
    apps_config::{AppExtraFlag, SETTINGS_BY_APP},
    error_handler::Result,
    seelen::get_app_handle,
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

#[derive(Getters, MutGetters)]
pub struct WindowManager {
    window: WebviewWindow,
    tiled_handles: Vec<isize>,
    floating_handles: Vec<isize>,
    pub current_virtual_desktop: String,
    paused: bool,
    #[getset(get = "pub")]
    ready: bool,
}

impl WindowManager {
    pub const TARGET: &'static str = "window-manager";
    pub const VIRTUAL_PREVIEWS: [&'static str; 2] = [
        "Virtual desktop switching preview",
        "Virtual desktop hotkey switching preview",
    ];

    pub fn new(monitor: isize) -> Result<Self> {
        log::info!("Creating Tiling Windows Manager / {}", monitor);

        let handle = get_app_handle();

        Ok(Self {
            window: Self::create_window(&handle, monitor)?,
            tiled_handles: Vec::new(),
            floating_handles: Vec::new(),
            current_virtual_desktop: VirtualDesktopManager::get_current_virtual_desktop()?.id(),
            paused: true, // paused until complete_window_setup is called
            ready: false,
        })
    }

    pub fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        self.window.emit_to(self.window.label(), event, payload)?;
        Ok(())
    }

    pub fn complete_window_setup(&mut self) -> Result<()> {
        log::info!("Tiling Windows Manager Created");
        self.paused = false;
        self.ready = true;
        self.window
            .emit("set-active-workspace", &self.current_virtual_desktop)?;
        Ok(())
    }

    pub fn is_managed(&self, hwnd: HWND) -> bool {
        self.tiled_handles.contains(&hwnd.0) || self.floating_handles.contains(&hwnd.0)
    }

    pub fn is_floating(&self, hwnd: HWND) -> bool {
        self.floating_handles.contains(&hwnd.0)
    }

    pub fn set_active_window(&mut self, hwnd: HWND) -> Result<()> {
        if WindowsApi::get_window_text(hwnd) == "Task Switching" {
            return Ok(());
        }

        log::trace!(
            "Setting active window to {} <=> {:?}",
            hwnd.0,
            WindowsApi::get_window_text(hwnd),
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
        self.emit("set-active-window", hwnd.0)?;
        Ok(())
    }

    pub fn set_active_workspace(&mut self, virtual_desktop_id: String) -> Result<()> {
        if virtual_desktop_id == self.current_virtual_desktop {
            return Ok(());
        }
        log::trace!("Setting active workspace to: {}", virtual_desktop_id);
        self.current_virtual_desktop = virtual_desktop_id;
        self.window
            .emit("set-active-workspace", &self.current_virtual_desktop)?;
        Ok(())
    }

    pub fn add_hwnd(&mut self, hwnd: HWND) -> Result<bool> {
        if self.paused || self.is_managed(hwnd) {
            return Ok(false);
        }

        let mut desktop_to_add = self.current_virtual_desktop.clone();
        if WindowsApi::is_cloaked(hwnd)? {
            desktop_to_add = VirtualDesktopManager::get_by_window(hwnd)?.id();
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

        self.emit(
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
        self.emit(
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
        self.emit("remove-window", hwnd.0)?;
        Ok(true)
    }

    pub fn force_retiling(&self) -> Result<()> {
        log::trace!("Forcing retiling");
        self.emit("force-retiling", ())?;
        Ok(())
    }

    pub fn pseudo_pause(&self) -> Result<()> {
        WindowsApi::bring_to(self.window.hwnd()?, HWND_BOTTOM)
    }

    pub fn pseudo_resume(&self) -> Result<()> {
        WindowsApi::bring_to(self.window.hwnd()?, HWND_TOPMOST)
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

    fn create_window(handle: &AppHandle<Wry>, monitor_id: isize) -> Result<WebviewWindow> {
        let monitor_info = WindowsApi::monitor_info(HMONITOR(monitor_id))?;
        let work_area = monitor_info.monitorInfo.rcWork;

        let window = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            handle,
            format!("{}/{}", Self::TARGET, monitor_id),
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

        WindowsApi::set_position(
            window.hwnd()?,
            Some(HWND_TOPMOST),
            &work_area,
            SWP_NOACTIVATE,
        )?;

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
