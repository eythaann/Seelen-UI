pub mod hook;
pub mod instance;
pub mod node_impl;
pub mod state;

use instance::WindowManagerV2;
use seelen_core::{handlers::SeelenEvent, state::AppExtraFlag};
use state::WM_STATE;
use tauri::Emitter;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{WS_CAPTION, WS_EX_TOPMOST},
};

use crate::{
    error_handler::Result,
    log_error,
    modules::virtual_desk::get_vd_manager,
    seelen::get_app_handle,
    seelen_weg::SeelenWeg,
    state::application::FULL_STATE,
    trace_lock,
    windows_api::{window::Window, WindowEnumerator, WindowsApi},
};

impl WindowManagerV2 {
    fn is_manageable_window(hwnd: HWND) -> bool {
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

    fn is_managed(window: &Window) -> bool {
        trace_lock!(WM_STATE).contains(window)
    }

    fn force_retiling() -> Result {
        get_app_handle().emit(SeelenEvent::WMForceRetiling, ())?;
        Ok(())
    }

    fn set_overlay_visibility(visible: bool) -> Result {
        get_app_handle().emit(SeelenEvent::WMSetOverlayVisibility, visible)?;
        Ok(())
    }

    fn set_active_window(window: &Window) -> Result {
        get_app_handle().emit(SeelenEvent::WMSetActiveWindow, window.address())?;
        Ok(())
    }

    fn add(window: &Window) -> Result<()> {
        let mut state = trace_lock!(WM_STATE);
        let monitor_id = window.monitor().id()?;

        let vd_manager = get_vd_manager();
        let workspace_id = if vd_manager.uses_cloak() && window.is_cloaked() {
            window.workspace()?.id()
        } else {
            vd_manager.get_current()?.id()
        };

        if let Some(m) = state.get_monitor_mut(&monitor_id) {
            if let Some(w) = m.get_workspace_mut(&workspace_id) {
                w.add_window(window);
                get_app_handle().emit_to(
                    format!("{}/{}", Self::TARGET, monitor_id),
                    SeelenEvent::WMSetLayout,
                    w.get_root_node().map(|n| n.inner()),
                )?;
            }
        }
        Ok(())
    }

    fn remove(window: &Window) -> Result<()> {
        let mut state = trace_lock!(WM_STATE);
        let current_workspace = get_vd_manager().get_current()?.id();

        for (monitor_id, monitor) in state.monitors.iter_mut() {
            for (workspace_id, workspace) in monitor.workspaces.iter_mut() {
                workspace.remove_window(window);
                if workspace_id == &current_workspace {
                    get_app_handle().emit_to(
                        format!("{}/{}", Self::TARGET, monitor_id),
                        SeelenEvent::WMSetLayout,
                        workspace.get_root_node().map(|n| n.inner()),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn enumerate_all_windows() -> Result<()> {
        WindowEnumerator::new().for_each(|hwnd| {
            if Self::should_be_managed(hwnd) {
                log_error!(Self::add(&Window::from(hwnd)));
            }
        })
    }
}
