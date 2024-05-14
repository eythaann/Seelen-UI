pub mod cli;
pub mod hook;

use crate::{
    error_handler::Result,
    seelen::{get_app_handle, APP_STATE},
    windows_api::WindowsApi,
};
use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::HMONITOR,
    UI::{
        Shell::{SHAppBarMessage, ABE_TOP, ABM_NEW, ABM_SETPOS, APPBARDATA},
        WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SWP_NOSIZE},
    },
};

pub struct FancyToolbar {
    window: WebviewWindow,
    hitbox_window: WebviewWindow,
    // -- -- -- --
    last_focus: Option<isize>,
}

#[derive(Serialize, Clone)]
pub struct ActiveApp {
    title: String,
    name: String,
}

impl FancyToolbar {
    pub fn new(monitor: isize) -> Result<Self> {
        log::info!("Creating Fancy Toolbar");
        let handle = get_app_handle();
        let (window, hitbox_window) = Self::create_window(&handle, monitor)?;
        Ok(Self {
            window,
            hitbox_window,
            last_focus: None,
        })
    }

    pub fn focus_changed(&mut self, hwnd: HWND) -> Result<()> {
        let title = WindowsApi::get_window_text(hwnd);

        self.last_focus = Some(hwnd.0);
        self.window.emit(
            "focus-changed",
            ActiveApp {
                title,
                name: WindowsApi::get_window_display_name(hwnd)
                    .unwrap_or(String::from("Error on App Name")),
            },
        )?;
        Ok(())
    }

    pub fn ensure_hitbox_zorder(&self) -> Result<()> {
        WindowsApi::bring_to(
            HWND(self.hitbox_window.hwnd()?.0),
            HWND(self.window.hwnd()?.0),
        )
    }
}

// statics
impl FancyToolbar {
    const TARGET: &'static str = "fancy-toolbar";
    const TARGET_HITBOX: &'static str = "fancy-toolbar-hitbox";

    fn create_window(
        manager: &AppHandle<Wry>,
        monitor: isize,
    ) -> Result<(WebviewWindow, WebviewWindow)> {
        let monitor_info = WindowsApi::monitor_info(HMONITOR(monitor))?;
        let rc_monitor = monitor_info.monitorInfo.rcMonitor;

        let hitbox = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            manager,
            format!("{}/{}", Self::TARGET_HITBOX, monitor),
            tauri::WebviewUrl::App("toolbar-hitbox/index.html".into()),
        )
        .title("Seelen Fancy Toolbar Hitbox")
        .maximizable(false)
        .minimizable(false)
        .resizable(false)
        .visible(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()?;

        let window = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            manager,
            format!("{}/{}", Self::TARGET, monitor),
            tauri::WebviewUrl::App("toolbar/index.html".into()),
        )
        .title("Seelen Fancy Toolbar")
        .maximizable(false)
        .minimizable(false)
        .resizable(false)
        .visible(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()?;

        window.set_ignore_cursor_events(true)?;

        let main_hwnd = HWND(window.hwnd()?.0);
        let hitbox_hwnd = HWND(hitbox.hwnd()?.0);

        // pre set position for resize in case of multiples dpi
        WindowsApi::set_position(main_hwnd, None, &rc_monitor, SWP_NOSIZE)?;
        WindowsApi::set_position(hitbox_hwnd, None, &rc_monitor, SWP_NOSIZE)?;

        let mut rect = rc_monitor.clone();
        rect.bottom = rect.bottom - 1; // avoid be matched as a fullscreen app;

        let dpi = WindowsApi::get_device_pixel_ratio(HMONITOR(monitor))?;
        let toolbar_height = APP_STATE.lock().get_toolbar_height();

        let mut abd = APPBARDATA::default();
        abd.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
        abd.hWnd = hitbox_hwnd;
        abd.uEdge = ABE_TOP;

        abd.rc = rc_monitor.clone();
        abd.rc.bottom = abd.rc.top + (toolbar_height as f32 * dpi) as i32;

        unsafe {
            SHAppBarMessage(ABM_NEW, &mut abd);
            SHAppBarMessage(ABM_SETPOS, &mut abd);
        }

        WindowsApi::set_position(hitbox_hwnd, None, &abd.rc, SWP_ASYNCWINDOWPOS)?;
        WindowsApi::set_position(main_hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;

        log::info!("Fancy Toolbar setup completed for {}", monitor);

        Ok((window, hitbox))
    }
}
