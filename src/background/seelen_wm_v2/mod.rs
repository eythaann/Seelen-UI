pub mod cli;
pub mod handler;
pub mod hook;
pub mod instance;
pub mod node_impl;
pub mod state;

use instance::WindowManagerV2;
use seelen_core::{
    handlers::SeelenEvent,
    state::{AppExtraFlag, WorkspaceId},
    system_state::MonitorId,
};
use state::{WmV2StateWorkspace, WM_STATE};
use tauri::Emitter;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{WS_CAPTION, WS_EX_TOPMOST},
};

use crate::{
    error_handler::Result,
    log_error,
    seelen::get_app_handle,
    state::application::FULL_STATE,
    trace_lock,
    virtual_desktops::get_vd_manager,
    windows_api::{window::Window, WindowEnumerator, WindowsApi},
};

impl WindowManagerV2 {
    fn is_manageable_window(hwnd: HWND) -> bool {
        let window = Window::from(hwnd);
        let exe = window.process().program_path();

        if let Ok(exe) = &exe {
            if exe.ends_with("ApplicationFrameHost.exe") && window.is_real_window() {
                return true;
            }
        }

        // Without admin some apps does not return the exe path so these should be unmanaged
        exe.is_ok()
        && window.is_real_window()
        // Ignore windows without a title bar, and top most windows normally are widgets or tools so they should not be managed
        && (WindowsApi::get_styles(hwnd).contains(WS_CAPTION) && !WindowsApi::get_ex_styles(hwnd).contains(WS_EX_TOPMOST))
        && !window.is_minimized()
        && (get_vd_manager().uses_cloak() || !window.is_cloaked())
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

    pub fn force_retiling() -> Result<()> {
        get_app_handle().emit(SeelenEvent::WMForceRetiling, ())?;
        Ok(())
    }

    fn render_workspace(monitor_id: &str, w: &WmV2StateWorkspace) -> Result<()> {
        get_app_handle().emit_to(
            Self::get_label(monitor_id),
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
        window.init_cache();

        let monitor_id = window.monitor().stable_id2()?;

        let mut vd_manager = get_vd_manager();
        let vd_active_id = vd_manager.get_active_workspace_id(&monitor_id).clone();

        let win_vd_workspace = vd_manager
            .workspace_containing_window(&window.address())
            .ok_or("Window is not associated with any workspace")?;

        // TODO: Reimplement this
        /* if let Some(config) = FULL_STATE.load().get_app_config_by_window(window.hwnd()) {
            if let Some(index) = config.bound_monitor {
                if let Some(monitor) = Monitor::at(index) {
                    monitor_id = monitor.stable_id()?;
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
        } */

        let mut state = trace_lock!(WM_STATE);
        if let Some(monitor) = state.get_monitor_mut(&monitor_id) {
            let wm_workspace = monitor.get_workspace_mut(&win_vd_workspace.id);
            wm_workspace.add_window(window);

            if win_vd_workspace.id == vd_active_id {
                get_app_handle().emit_to(
                    Self::get_label(&monitor_id),
                    SeelenEvent::WMSetLayout,
                    wm_workspace.get_root_node(),
                )?;
            }
        }
        Ok(())
    }

    fn remove(window: &Window) -> Result<()> {
        let monitor_id = window.get_cached_data().monitor;
        let mut state = trace_lock!(WM_STATE);
        let Some(monitor) = state.get_monitor_mut(&monitor_id) else {
            return Ok(());
        };

        let mut vd = get_vd_manager();
        let current_workspace = vd.get_active_workspace_id(&monitor_id);

        for (workspace_id, workspace) in monitor.workspaces.iter_mut() {
            workspace.remove_window(window);

            if workspace_id == current_workspace {
                get_app_handle().emit_to(
                    Self::get_label(&monitor_id),
                    SeelenEvent::WMSetLayout,
                    workspace.get_root_node(),
                )?;
            }
        }
        Ok(())
    }

    fn workspace_changed(monitor_id: &MonitorId, workspace_id: &WorkspaceId) -> Result<()> {
        let mut state = trace_lock!(WM_STATE);

        let monitor = state
            .get_monitor_mut(monitor_id)
            .ok_or("Monitor not found")?;
        let workspace = monitor.get_workspace_mut(workspace_id);

        get_app_handle().emit_to(
            Self::get_label(monitor_id),
            SeelenEvent::WMSetLayout,
            workspace.get_root_node(),
        )?;
        Ok(())
    }

    pub fn clear_state() {
        trace_lock!(WM_STATE).monitors.clear();
    }

    pub fn init_state() -> Result<()> {
        trace_lock!(WM_STATE).init()
    }

    pub fn enumerate_all_windows() -> Result<()> {
        WindowEnumerator::new().for_each(|window| {
            if !Self::is_managed(&window) && Self::should_be_managed(window.hwnd()) {
                log_error!(Self::add(&window));
            }
        })
    }
}
