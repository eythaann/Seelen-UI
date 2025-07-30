use base64::Engine;
use getset::{Getters, MutGetters};
use seelen_core::handlers::SeelenEvent;
use std::sync::Arc;
use tauri::{Emitter, Listener, WebviewWindow};
use windows::Win32::{
    Foundation::HWND, Graphics::Gdi::HMONITOR, UI::WindowsAndMessaging::SWP_ASYNCWINDOWPOS,
};

use crate::{
    error_handler::Result,
    log_error,
    modules::virtual_desk::{get_vd_manager, VirtualDesktopManagerTrait},
    seelen::get_app_handle,
    seelen_bar::FancyToolbar,
    seelen_wm_v2::state::WM_STATE,
    trace_lock,
    windows_api::WindowsApi,
};

#[derive(Getters, MutGetters)]
pub struct WindowManagerV2 {
    pub window: WebviewWindow,
}

impl Drop for WindowManagerV2 {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        log_error!(self.window.destroy());
    }
}

impl WindowManagerV2 {
    pub const TITLE: &'static str = ".Seelen Window Manager";
    pub const TARGET: &'static str = "@seelen/window-manager";

    pub fn new(monitor_id: &str) -> Result<Self> {
        Ok(Self {
            window: Self::create_window(monitor_id)?,
        })
    }

    pub fn hwnd(&self) -> Result<HWND> {
        Ok(HWND(self.window.hwnd()?.0))
    }

    pub fn get_label(monitor_id: &str) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!(
            "{}?monitorId={}",
            Self::TARGET,
            monitor_id
        ))
    }

    fn create_window(monitor_id: &str) -> Result<WebviewWindow> {
        let label = format!("{}?monitorId={}", Self::TARGET, monitor_id);
        log::info!("Creating {label}");
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&label);

        let window = tauri::WebviewWindowBuilder::new(
            get_app_handle(),
            label,
            tauri::WebviewUrl::App("seelen_wm_v2/index.html".into()),
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
        .always_on_top(true)
        .build()?;

        window.set_ignore_cursor_events(true)?;

        let window_label = Arc::new(window.label().to_owned());
        let monitor_id = Arc::new(monitor_id.to_owned());
        window.listen("complete-setup", move |_event| {
            let window_label = window_label.clone();
            let monitor_id = monitor_id.clone();
            std::thread::spawn(move || -> Result<()> {
                let app = get_app_handle();
                let mut state = trace_lock!(WM_STATE);

                if let Some(m) = state.get_monitor_mut(&monitor_id) {
                    let workspace_id = get_vd_manager().get_current()?.id();
                    let w = m.get_workspace_mut(&workspace_id);
                    app.emit_to(
                        window_label.as_ref(),
                        SeelenEvent::WMSetLayout,
                        w.get_root_node(),
                    )?;
                }

                app.emit(
                    SeelenEvent::WMSetActiveWindow,
                    WindowsApi::get_foreground_window().0 as isize,
                )?;
                Ok(())
            });
        });

        Ok(window)
    }

    pub fn set_position(&self, monitor: HMONITOR) -> Result<()> {
        let work_area = FancyToolbar::get_work_area_by_monitor(monitor)?;
        let main_hwnd = self.hwnd()?;
        WindowsApi::move_window(main_hwnd, &work_area)?;
        WindowsApi::set_position(main_hwnd, None, &work_area, SWP_ASYNCWINDOWPOS)?;
        Ok(())
    }
}
