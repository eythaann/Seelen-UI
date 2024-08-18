pub mod cli;
pub mod hook;

use crate::{
    error_handler::Result,
    log_error,
    seelen::get_app_handle,
    state::application::FULL_STATE,
    utils::is_virtual_desktop_supported,
    windows_api::{AppBarData, AppBarDataEdge, WindowsApi},
};
use serde::Serialize;
use tauri::{Emitter, Listener, Manager, WebviewWindow};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{HWND_TOPMOST, SWP_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE},
};

pub struct FancyToolbar {
    window: WebviewWindow,
    hitbox: WebviewWindow,
    // -- -- -- --
    last_focus: Option<isize>,
    hidden: bool,
}

#[derive(Serialize, Clone)]
pub struct ActiveApp {
    title: String,
    name: String,
    exe: Option<String>,
}

impl Drop for FancyToolbar {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        if let Ok(hwnd) = self.hitbox.hwnd() {
            AppBarData::from_handle(hwnd).unregister_bar();
        }
        log_error!(self.window.destroy());
        log_error!(self.hitbox.destroy());
    }
}

impl FancyToolbar {
    pub fn new(postfix: &str) -> Result<Self> {
        log::info!("Creating {}/{}", Self::TARGET, postfix);
        let (window, hitbox) = Self::create_window(postfix)?;
        Ok(Self {
            window,
            hitbox,
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
        WindowsApi::show_window_async(self.hitbox.hwnd()?, SW_HIDE)?;
        self.hidden = true;
        Ok(())
    }

    pub fn show(&mut self) -> Result<()> {
        WindowsApi::show_window_async(self.window.hwnd()?, SW_SHOWNOACTIVATE)?;
        WindowsApi::show_window_async(self.hitbox.hwnd()?, SW_SHOWNOACTIVATE)?;
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
                exe: WindowsApi::exe_path(hwnd).ok(),
            },
        )?;
        Ok(())
    }

    pub fn ensure_hitbox_zorder(&self) -> Result<()> {
        let hitbox = HWND(self.hitbox.hwnd()?.0);
        WindowsApi::bring_to(hitbox, HWND_TOPMOST)?;
        self.set_positions(WindowsApi::monitor_from_window(hitbox).0)?;
        Ok(())
    }
}

// statics
impl FancyToolbar {
    const TARGET: &'static str = "fancy-toolbar";
    const TARGET_HITBOX: &'static str = "fancy-toolbar-hitbox";

    /// Work area no works fine on multiple monitors
    /// so we use this functions that only takes the toolbar in account
    pub fn get_work_area_by_monitor(monitor: isize) -> Result<RECT> {
        let monitor_info = WindowsApi::monitor_info(HMONITOR(monitor))?;

        let dpi = WindowsApi::get_device_pixel_ratio(HMONITOR(monitor))?;
        let mut rect = monitor_info.monitorInfo.rcMonitor;

        let state = FULL_STATE.load();
        if state.is_bar_enabled() {
            let toolbar_height = state.settings().fancy_toolbar.height;
            rect.top += (toolbar_height as f32 * dpi) as i32;
        }

        Ok(rect)
    }

    pub fn set_positions(&self, monitor: isize) -> Result<()> {
        let hmonitor = HMONITOR(monitor);
        if hmonitor.is_invalid() {
            return Err("Invalid Monitor".into());
        }

        let monitor_info = WindowsApi::monitor_info(hmonitor)?;
        let rc_monitor = monitor_info.monitorInfo.rcMonitor;

        let main_hwnd = HWND(self.window.hwnd()?.0);
        let hitbox_hwnd = HWND(self.hitbox.hwnd()?.0);

        let dpi = WindowsApi::get_device_pixel_ratio(hmonitor)?;
        let toolbar_height = FULL_STATE.load().settings().fancy_toolbar.height;

        let mut abd = AppBarData::from_handle(hitbox_hwnd);

        let mut abd_rect = rc_monitor;
        abd_rect.bottom = abd_rect.top + (toolbar_height as f32 * dpi) as i32;

        abd.set_edge(AppBarDataEdge::Top);
        abd.set_rect(abd_rect);

        abd.register_as_new_bar();

        // pre set position for resize in case of multiples dpi
        WindowsApi::move_window(hitbox_hwnd, &rc_monitor)?;
        WindowsApi::set_position(hitbox_hwnd, None, &abd_rect, SWP_NOACTIVATE)?;

        WindowsApi::move_window(main_hwnd, &rc_monitor)?;
        WindowsApi::set_position(main_hwnd, None, &rc_monitor, SWP_NOACTIVATE)?;
        Ok(())
    }

    fn create_window(postfix: &str) -> Result<(WebviewWindow, WebviewWindow)> {
        let manager = get_app_handle();

        let label = format!("{}/{}", Self::TARGET_HITBOX, postfix);
        let hitbox = match manager.get_webview_window(&label) {
            Some(window) => window,
            None => tauri::WebviewWindowBuilder::new(
                &manager,
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
            .drag_and_drop(false)
            .build()?,
        };

        let label = format!("{}/{}", Self::TARGET, postfix);
        let window = match manager.get_webview_window(&label) {
            Some(window) => window,
            None => tauri::WebviewWindowBuilder::new(
                &manager,
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
            .drag_and_drop(false)
            .owner(&hitbox)?
            .build()?,
        };

        window.set_ignore_cursor_events(true)?;

        window.once("store-events-ready", Self::on_store_events_ready);
        Ok((window, hitbox))
    }

    fn on_store_events_ready(_: tauri::Event) {
        // TODO refactor this implementation
        if is_virtual_desktop_supported() {
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
}
