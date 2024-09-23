pub mod cli;
pub mod handler;
pub mod hook;
pub mod instance;
pub mod node_impl;
pub mod state;

use instance::WindowManagerV2;
use seelen_core::{handlers::SeelenEvent, state::AppExtraFlag};
use state::{WmV2StateWorkspace, WM_STATE};
use tauri::Emitter;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{WS_CAPTION, WS_EX_TOPMOST},
};

use crate::{
    error_handler::Result,
    log_error,
    modules::virtual_desk::{get_vd_manager, VirtualDesktop},
    seelen::get_app_handle,
    seelen_weg::SeelenWeg,
    state::application::FULL_STATE,
    trace_lock,
    windows_api::{monitor::Monitor, window::Window, WindowEnumerator, WindowsApi},
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

    fn force_retiling() -> Result<()> {
        get_app_handle().emit(SeelenEvent::WMForceRetiling, ())?;
        Ok(())
    }

    fn render_workspace(monitor_id: &str, w: &WmV2StateWorkspace) -> Result<()> {
        get_app_handle().emit_to(
            format!("{}/{}", Self::TARGET, monitor_id),
            SeelenEvent::WMSetLayout,
            w.get_root_node(),
        )?;
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
        let vd_manager = get_vd_manager();
        let current_workspace_id = vd_manager.get_current()?.id();

        let mut monitor_id = window.monitor().id()?;
        let mut workspace_id = window.workspace()?.id();

        if let Some(config) = FULL_STATE.load().get_app_config_by_window(window.hwnd()) {
            if let Some(index) = config.bound_monitor {
                if let Some(monitor) = Monitor::at(index) {
                    monitor_id = monitor.id()?;
                }
            }
            if let Some(index) = config.bound_workspace {
                let addr = window.address();
                vd_manager.send_to(index, addr)?;
                std::thread::sleep(std::time::Duration::from_millis(20));
                vd_manager.switch_to(index)?;
                if let Some(workspace) = vd_manager.get(index)? {
                    workspace_id = workspace.id();
                }
            }
        }

        if let Some(monitor) = state.get_monitor_mut(&monitor_id) {
            let workspace = monitor.get_workspace_mut(&workspace_id);
            workspace.add_window(window);
            if workspace_id == current_workspace_id {
                get_app_handle().emit_to(
                    format!("{}/{}", Self::TARGET, monitor_id),
                    SeelenEvent::WMSetLayout,
                    workspace.get_root_node(),
                )?;
            }
        }
        Ok(())
    }

    fn remove(window: &Window) -> Result<()> {
        let mut state = trace_lock!(WM_STATE);
        let current_workspace = get_vd_manager().get_current()?.id();

        // TODO this can be optimized, later
        for (monitor_id, monitor) in state.monitors.iter_mut() {
            for (workspace_id, workspace) in monitor.workspaces.iter_mut() {
                workspace.remove_window(window);
                if workspace_id == &current_workspace {
                    get_app_handle().emit_to(
                        format!("{}/{}", Self::TARGET, monitor_id),
                        SeelenEvent::WMSetLayout,
                        workspace.get_root_node(),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn workspace_changed(current: &VirtualDesktop) -> Result<()> {
        let mut state = trace_lock!(WM_STATE);
        let workspace_id = current.id();
        for (monitor_id, monitor) in state.monitors.iter_mut() {
            let workspace = monitor.get_workspace_mut(&workspace_id);
            get_app_handle().emit_to(
                format!("{}/{}", Self::TARGET, monitor_id),
                SeelenEvent::WMSetLayout,
                workspace.get_root_node(),
            )?;
        }
        Ok(())
    }

    pub fn clear_state() {
        trace_lock!(WM_STATE).monitors.clear();
    }

    pub fn init_state() -> Result<()> {
        trace_lock!(WM_STATE).init()
    }

    pub fn enumerate_all_windows() -> Result<()> {
        WindowEnumerator::new().for_each(|hwnd| {
            let window = Window::from(hwnd);
            if !Self::is_managed(&window) && Self::should_be_managed(hwnd) {
                log_error!(Self::add(&window));
            }
        })
    }
}
