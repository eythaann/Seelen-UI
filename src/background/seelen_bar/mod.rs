pub mod cli;
pub mod hook;

use crate::{
    error_handler::Result,
    log_error,
    modules::virtual_desk::get_vd_manager,
    seelen::get_app_handle,
    state::application::FULL_STATE,
    utils::are_overlaped,
    windows_api::{AppBarData, AppBarDataEdge, WindowsApi},
};
use itertools::Itertools;
use seelen_core::state::HideMode;
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
    pub cached_monitor: HMONITOR,
    last_focus: Option<isize>,
    hidden: bool,
    overlaped: bool,
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
            cached_monitor: HMONITOR(-1),
            overlaped: false,
        })
    }

    pub fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        self.window.emit_to(self.window.label(), event, payload)?;
        Ok(())
    }

    pub fn is_overlapping(&self, hwnd: HWND) -> Result<bool> {
        let rect = WindowsApi::get_window_rect_without_margins(hwnd);
        let monitor_info = WindowsApi::monitor_info(self.cached_monitor)?;
        let dpi = WindowsApi::get_device_pixel_ratio(self.cached_monitor)?;
        let height = FULL_STATE.load().settings().fancy_toolbar.height;

        let mut hitbox_rect = monitor_info.monitorInfo.rcMonitor;
        hitbox_rect.bottom = hitbox_rect.top + (height as f32 * dpi) as i32;
        Ok(are_overlaped(&hitbox_rect, &rect))
    }

    pub fn set_overlaped_status(&mut self, is_overlaped: bool) -> Result<()> {
        if self.overlaped == is_overlaped {
            return Ok(());
        }
        self.overlaped = is_overlaped;
        self.set_positions(self.cached_monitor.0)?;
        self.emit("set-auto-hide", self.overlaped)?;
        Ok(())
    }

    pub fn handle_overlaped_status(&mut self, hwnd: HWND) -> Result<()> {
        if !WindowsApi::is_window_visible(hwnd) {
            return Ok(());
        }
        self.set_overlaped_status(self.is_overlapping(hwnd)?)
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

        let state = FULL_STATE.load();
        let settings = &state.settings().fancy_toolbar;
        let dpi = WindowsApi::get_device_pixel_ratio(hmonitor)?;

        let mut abd = AppBarData::from_handle(hitbox_hwnd);
        if settings.hide_mode != HideMode::Never {
            abd.unregister_bar();
        }

        let mut abd_rect = rc_monitor;
        abd_rect.bottom = if settings.hide_mode == HideMode::Always
            || (self.overlaped && settings.hide_mode == HideMode::OnOverlap)
        {
            abd_rect.top + 1
        } else {
            abd_rect.top + (settings.height as f32 * dpi) as i32
        };

        if settings.hide_mode == HideMode::Never {
            abd.set_edge(AppBarDataEdge::Top);
            abd.set_rect(abd_rect);
            abd.register_as_new_bar();
        }

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
        std::thread::spawn(|| -> Result<()> {
            let handler = get_app_handle();
            let vd = get_vd_manager();
            let desktops = vd
                .get_all()?
                .iter()
                .map(|d| d.as_serializable())
                .collect_vec();
            handler.emit("workspaces-changed", &desktops)?;
            handler.emit("active-workspace-changed", vd.get_current()?.id())?;
            Ok(())
        });
    }
}
