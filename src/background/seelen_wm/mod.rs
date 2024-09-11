pub mod cli;
pub mod handler;
pub mod hook;

use std::sync::atomic::{AtomicIsize, Ordering};

use getset::{Getters, MutGetters};
use seelen_core::handlers::SeelenEvent;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Listener, WebviewWindow, Wry};
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{
        EnumWindows, HWND_BOTTOM, HWND_TOPMOST, SWP_NOACTIVATE, WS_CAPTION, WS_EX_TOPMOST,
    },
};

use crate::{
    error_handler::Result,
    log_error,
    modules::virtual_desk::get_vd_manager,
    seelen::{get_app_handle, SEELEN},
    seelen_bar::FancyToolbar,
    seelen_weg::SeelenWeg,
    state::{application::FULL_STATE, domain::AppExtraFlag},
    trace_lock,
    windows_api::WindowsApi,
};

#[derive(Serialize, Clone)]
pub struct ManagingApp {
    hwnd: isize,
    monitor: String,
    desktop_id: String,
    is_floating: bool,
}

#[derive(Getters, MutGetters)]
pub struct WindowManager {
    window: WebviewWindow,
    monitor: HMONITOR,
    apps: Vec<ManagingApp>,
    pub current_virtual_desktop: String,
    paused: bool,
    #[getset(get = "pub")]
    ready: bool,
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        log_error!(self.window.destroy());
    }
}

impl WindowManager {
    pub const TITLE: &'static str = "Seelen Window Manager";
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
            monitor: HMONITOR(monitor),
            apps: Vec::new(),
            current_virtual_desktop: get_vd_manager().get_current()?.id(),
            paused: true, // paused until complete-setup is called
            ready: false,
        })
    }

    pub fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        self.window.emit_to(self.window.label(), event, payload)?;
        Ok(())
    }

    pub fn is_managed(&self, hwnd: HWND) -> bool {
        self.get_app(hwnd).is_some()
    }

    pub fn is_floating(&self, hwnd: HWND) -> bool {
        self.get_app(hwnd)
            .map(|app| app.is_floating)
            .unwrap_or(false)
    }

    pub fn get_app(&self, hwnd: HWND) -> Option<&ManagingApp> {
        self.apps.iter().find(|app| app.hwnd == hwnd.0)
    }

    pub fn get_app_mut(&mut self, hwnd: HWND) -> Option<&mut ManagingApp> {
        self.apps.iter_mut().find(|app| app.hwnd == hwnd.0)
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
        self.emit(SeelenEvent::WMSetActiveWindow, hwnd.0)?;
        Ok(())
    }

    pub fn set_active_workspace(&mut self, virtual_desktop_id: String) -> Result<()> {
        if virtual_desktop_id == self.current_virtual_desktop {
            return Ok(());
        }
        log::trace!("Setting active workspace to: {}", virtual_desktop_id);
        self.current_virtual_desktop = virtual_desktop_id;
        self.window.emit(
            SeelenEvent::WMSetActiveWorkspace,
            &self.current_virtual_desktop,
        )?;
        Ok(())
    }

    pub fn add_hwnd(&mut self, hwnd: HWND) -> Result<bool> {
        if self.paused || self.is_managed(hwnd) {
            return Ok(false);
        }

        let desktop_to_add = if WindowsApi::is_cloaked(hwnd)? {
            get_vd_manager().get_by_window(hwnd.0)?.id()
        } else {
            self.current_virtual_desktop.clone()
        };

        log::trace!(
            "Adding {}({}) <=> {} on desktop: {}",
            WindowsApi::exe(hwnd).unwrap_or_default(),
            hwnd.0,
            WindowsApi::get_window_text(hwnd),
            desktop_to_add
        );

        let mut is_floating = false;
        if let Some(config) = FULL_STATE.load().get_app_config_by_window(hwnd) {
            is_floating = config.options.contains(&AppExtraFlag::Float);
        }

        let app = ManagingApp {
            hwnd: hwnd.0,
            monitor: WindowsApi::monitor_name(WindowsApi::monitor_from_window(hwnd))?,
            desktop_id: desktop_to_add,
            is_floating,
        };

        self.emit(SeelenEvent::WMAddWindow, &app)?;
        self.apps.push(app);
        Ok(true)
    }

    pub fn update_app(&mut self, hwnd: HWND) -> Result<()> {
        if self.paused {
            return Ok(());
        }
        let app = {
            let app = match self.get_app_mut(hwnd) {
                Some(app) => app,
                None => return Ok(()),
            };

            let current_desktop = get_vd_manager().get_by_window(app.hwnd)?.id();
            if app.desktop_id != current_desktop {
                app.desktop_id = current_desktop;
            }
            app.clone()
        };
        self.emit(SeelenEvent::WMUpdateWindow, app)?;
        Ok(())
    }

    /** triggered when a window is bounced by the front-end on adding action */
    pub fn bounce_handle(&mut self, hwnd: HWND) {
        if let Some(app) = self.get_app_mut(hwnd) {
            app.is_floating = true;
        }
    }

    fn remove_hwnd_no_emit(&mut self, hwnd: HWND) -> bool {
        if self.paused || !self.is_managed(hwnd) {
            return false;
        }
        self.apps.retain(|x| x.hwnd != hwnd.0);
        true
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
        self.emit(SeelenEvent::WMRemoveWindow, hwnd.0)?;
        Ok(true)
    }

    pub fn force_retiling(&self) -> Result<()> {
        log::trace!("Forcing retiling");
        self.emit(SeelenEvent::WMForceRetiling, ())?;
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

    pub fn should_be_added(&self, hwnd: HWND) -> bool {
        !self.is_managed(hwnd)
            && self.monitor == WindowsApi::monitor_from_window(hwnd)
            && Self::should_be_managed(hwnd)
    }
}

// UTILS AND STATICS
impl WindowManager {
    fn should_be_managed(hwnd: HWND) -> bool {
        if let Some(config) = FULL_STATE.load().get_app_config_by_window(hwnd) {
            if config.options.contains(&AppExtraFlag::Force) {
                return true;
            }

            if config.options.contains(&AppExtraFlag::Unmanage)
                || config.options.contains(&AppExtraFlag::Pinned)
            {
                return false;
            }
        }
        Self::is_manageable_window(hwnd)
    }

    pub fn is_manageable_window(hwnd: HWND) -> bool {
        let exe = WindowsApi::exe(hwnd);

        if let Ok(exe) = &exe {
            if exe.ends_with("ApplicationFrameHost.exe") && SeelenWeg::should_be_added(hwnd) {
                return true;
            }
        }

        // Without admin some apps does not return the exe path so these should be unmanaged
        exe.is_ok()
        && SeelenWeg::should_be_added(hwnd)
        // Ignore windows without a title bar, and top most windows normally are widgets or tools so they should not be managed
        && (WindowsApi::get_styles(hwnd).contains(WS_CAPTION) && !WindowsApi::get_ex_styles(hwnd).contains(WS_EX_TOPMOST))
        && !WindowsApi::is_iconic(hwnd)
        && (get_vd_manager().uses_cloak() || !WindowsApi::is_cloaked(hwnd).unwrap_or(false))
    }

    fn create_window(handle: &AppHandle<Wry>, monitor_id: isize) -> Result<WebviewWindow> {
        let work_area = FancyToolbar::get_work_area_by_monitor(monitor_id)?;

        let window = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            handle,
            format!("{}/{}", Self::TARGET, monitor_id),
            tauri::WebviewUrl::App("seelen_wm/index.html".into()),
        )
        .title(Self::TITLE)
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .resizable(false)
        .visible(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .drag_and_drop(false)
        .build()?;

        window.set_ignore_cursor_events(true)?;

        let main_hwnd = HWND(window.hwnd()?.0);
        WindowsApi::move_window(main_hwnd, &work_area)?;
        WindowsApi::set_position(main_hwnd, Some(HWND_TOPMOST), &work_area, SWP_NOACTIVATE)?;

        window.once("complete-setup", move |_event| {
            std::thread::spawn(move || -> Result<()> {
                if let Some(monitor) = trace_lock!(SEELEN).monitor_by_id_mut(monitor_id) {
                    if let Some(wm) = monitor.wm_mut() {
                        wm.paused = false;
                        wm.window.emit(
                            SeelenEvent::WMSetActiveWorkspace,
                            &wm.current_virtual_desktop,
                        )?;
                    }
                }
                Ok(())
            });
        });

        Ok(window)
    }

    unsafe extern "system" fn get_next_by_order_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // TODO search a way to handle ApplicationFrameHost.exe as well on change of virtual desktop
        if WindowManager::is_manageable_window(hwnd)
            && hwnd.0 != lparam.0
            && !WindowsApi::exe(hwnd).is_ok_and(|exe| &exe == "ApplicationFrameHost.exe")
        {
            NEXT.store(hwnd.0, Ordering::SeqCst);
            return false.into();
        }
        true.into()
    }

    pub fn get_next_by_order(hwnd: HWND) -> Option<HWND> {
        NEXT.store(0, Ordering::SeqCst);
        unsafe { EnumWindows(Some(Self::get_next_by_order_proc), LPARAM(hwnd.0)) }.ok();
        let result = NEXT.load(Ordering::SeqCst);
        if result == 0 {
            return None;
        }
        Some(HWND(result))
    }
}

static NEXT: AtomicIsize = AtomicIsize::new(0);
