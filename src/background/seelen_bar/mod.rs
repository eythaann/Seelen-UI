pub mod cli;
pub mod hook;

use std::sync::atomic::Ordering;

use crate::{
    error_handler::Result,
    seelen::get_app_handle,
    state::TOOLBAR_HEIGHT,
    windows_api::{AppBarData, AppBarDataEdge, WindowsApi},
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Listener, Manager, WebviewWindow, Wry};
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{
        HWND_TOPMOST, SWP_ASYNCWINDOWPOS, SWP_NOSIZE, SW_HIDE, SW_SHOWNOACTIVATE,
    },
};

pub struct FancyToolbar {
    window: WebviewWindow,
    #[allow(dead_code)]
    hitbox_window: WebviewWindow,
    // -- -- -- --
    last_focus: Option<isize>,
    hidden: bool,
}

#[derive(Serialize, Clone)]
pub struct ActiveApp {
    title: String,
    name: String,
}

impl FancyToolbar {
    pub fn new(monitor: isize) -> Result<Self> {
        log::info!("Creating Fancy Toolbar / {}", monitor);
        let handle = get_app_handle();
        let (window, hitbox_window) = Self::create_window(&handle, monitor)?;
        Ok(Self {
            window,
            hitbox_window,
            last_focus: None,
            hidden: false,
        })
    }

    pub fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        self.window.emit_to(self.window.label(), event, payload)?;
        Ok(())
    }

    pub fn hide(&mut self) -> Result<()> {
        WindowsApi::show_window_async(self.window.hwnd()?, SW_HIDE)?;
        WindowsApi::show_window_async(self.hitbox_window.hwnd()?, SW_HIDE)?;
        self.hidden = true;
        Ok(())
    }

    pub fn show(&mut self) -> Result<()> {
        WindowsApi::show_window_async(self.window.hwnd()?, SW_SHOWNOACTIVATE)?;
        WindowsApi::show_window_async(self.hitbox_window.hwnd()?, SW_SHOWNOACTIVATE)?;
        self.hidden = false;
        Ok(())
    }

    pub fn focus_changed(&mut self, hwnd: HWND) -> Result<()> {
        let title = WindowsApi::get_window_text(hwnd);
        self.last_focus = Some(hwnd.0);
        self.emit(
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
        WindowsApi::bring_to(HWND(self.hitbox_window.hwnd()?.0), HWND_TOPMOST)
    }
}

// statics
impl FancyToolbar {
    const TARGET: &'static str = "fancy-toolbar";
    const TARGET_HITBOX: &'static str = "fancy-toolbar-hitbox";

    fn set_positions(window: &WebviewWindow, hitbox: &WebviewWindow, monitor: isize) -> Result<()> {
        let monitor_info = WindowsApi::monitor_info(HMONITOR(monitor))?;
        let rc_monitor = monitor_info.monitorInfo.rcMonitor;
        let main_hwnd = HWND(window.hwnd()?.0);
        let hitbox_hwnd = HWND(hitbox.hwnd()?.0);

        // pre set position for resize in case of multiples dpi
        WindowsApi::set_position(main_hwnd, None, &rc_monitor, SWP_NOSIZE)?;
        WindowsApi::set_position(hitbox_hwnd, None, &rc_monitor, SWP_NOSIZE)?;

        {
            let dpi = WindowsApi::get_device_pixel_ratio(HMONITOR(monitor))?;
            let toolbar_height = TOOLBAR_HEIGHT.load(Ordering::Acquire);

            let mut abd = AppBarData::from_handle(hitbox_hwnd);

            let mut abd_rect = rc_monitor;
            abd_rect.bottom = abd_rect.top + (toolbar_height as f32 * dpi) as i32;

            abd.set_edge(AppBarDataEdge::Top);
            abd.set_rect(abd_rect);

            abd.register_as_new_bar();
            WindowsApi::set_position(hitbox_hwnd, None, &abd_rect, SWP_ASYNCWINDOWPOS)?;
        }

        let mut rect = rc_monitor;
        rect.bottom -= 1; // avoid be matched as a fullscreen app;
        WindowsApi::set_position(main_hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;
        Ok(())
    }

    fn create_window(
        manager: &AppHandle<Wry>,
        monitor: isize,
    ) -> Result<(WebviewWindow, WebviewWindow)> {
        let label = format!("{}/{}", Self::TARGET_HITBOX, monitor);

        let hitbox = match manager.get_webview_window(&label) {
            Some(window) => window,
            None => tauri::WebviewWindowBuilder::new(
                manager,
                label,
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
            .build()?,
        };

        let label = format!("{}/{}", Self::TARGET, monitor);
        let window = match manager.get_webview_window(&label) {
            Some(window) => window,
            None => tauri::WebviewWindowBuilder::new(
                manager,
                label,
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
            .owner(&hitbox)?
            .build()?,
        };

        window.set_ignore_cursor_events(true)?;
        Self::set_positions(&window, &hitbox, monitor)?;

        window.once("store-events-ready", Self::on_store_events_ready);
        Ok((window, hitbox))
    }

    fn on_store_events_ready(_: tauri::Event) {
        std::thread::spawn(|| -> Result<()> {
            let handler = get_app_handle();

            let desktops = winvd::get_desktops()?;
            let current_desktop = winvd::get_current_desktop()?;

            let mut desktops_names = Vec::new();
            for (i, d) in desktops.iter().enumerate() {
                if let Ok(name) = d.get_name() {
                    desktops_names.push(name);
                } else {
                    desktops_names.push(format!("Desktop {}", i + 1))
                }
            }

            handler.emit("workspaces-changed", desktops_names)?;
            handler.emit("active-workspace-changed", current_desktop.get_index()?)?;
            Ok(())
        });
    }
}
