pub mod cli;
pub mod handler;
pub mod hook;
pub mod instance;
pub mod node_ext;
pub mod state;

use instance::WindowManagerV2;
use seelen_core::{
    handlers::SeelenEvent,
    state::{AppExtraFlag, WorkspaceId},
    system_state::MonitorId,
};
use state::{WmWorkspaceState, WM_STATE};
use tauri::Emitter;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{WS_EX_TOPMOST, WS_SIZEBOX},
};

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    state::application::FULL_STATE,
    trace_lock,
    virtual_desktops::get_vd_manager,
    widgets::window_manager::node_ext::WmNodeExt,
    windows_api::{window::Window, WindowEnumerator, WindowsApi},
};

impl WindowManagerV2 {
    fn should_be_managed(hwnd: HWND) -> bool {
        let window = Window::from(hwnd);
        if !window.is_interactable_and_not_hidden() {
            return false;
        }

        if let Some(config) = FULL_STATE.load().get_app_config_by_window(hwnd) {
            if config.options.contains(&AppExtraFlag::VdPinned) {
                return false;
            }

            if config.options.contains(&AppExtraFlag::WmForce) {
                return true;
            }

            if config.options.contains(&AppExtraFlag::WmUnmanage) {
                return false;
            }
        }

        let styles = WindowsApi::get_styles(hwnd);
        // Ignore windows that are not resizable
        if !styles.contains(WS_SIZEBOX) {
            return false;
        }

        let ex_styles = WindowsApi::get_ex_styles(hwnd);
        // Top most windows normally are widgets or tools that should not be managed
        if ex_styles.contains(WS_EX_TOPMOST) {
            return false;
        }

        true
    }

    fn is_managed(window: &Window) -> bool {
        trace_lock!(WM_STATE).is_managed(window)
    }

    fn is_tiled(window: &Window) -> bool {
        trace_lock!(WM_STATE).is_tiled(window)
    }

    pub fn force_retiling() -> Result<()> {
        get_app_handle().emit(SeelenEvent::WMForceRetiling, ())?;
        Ok(())
    }

    fn render_workspace(monitor_id: &str, workspace: &WmWorkspaceState) -> Result<()> {
        log::trace!("Rendering workspace on {monitor_id}");
        workspace.layout.structure.hide_non_active()?;
        get_app_handle().emit_to(
            Self::get_label(monitor_id),
            SeelenEvent::WMSetLayout,
            &workspace.layout.structure,
        )?;
        Ok(())
    }

    fn add(window: &Window) -> Result<()> {
        let monitor_id = window.monitor_id();

        let mut vd_manager = get_vd_manager();
        let vd_active_id = vd_manager.get_active_workspace_id(&monitor_id).clone();

        let win_vd_workspace = vd_manager
            .workspace_containing_window(&window.address())
            .ok_or("Window is not associated with any workspace")?;

        let mut state = trace_lock!(WM_STATE);
        let wm_workspace = state.get_workspace_state(&win_vd_workspace.id);
        wm_workspace.add_to_tiles(window);

        if win_vd_workspace.id == vd_active_id {
            Self::render_workspace(&monitor_id, wm_workspace)?;
        }
        Ok(())
    }

    fn remove(window: &Window) -> Result<()> {
        let mut state = trace_lock!(WM_STATE);
        let vd = get_vd_manager();
        for (workspace_id, workspace) in &mut state.layouts {
            if workspace.is_managed(window) {
                workspace.unmanage(window);
                if let Some(monitor_id) = vd.monitor_containing_workspace(workspace_id) {
                    Self::render_workspace(&monitor_id, workspace)?;
                }
                break;
            }
        }
        Ok(())
    }

    fn workspace_changed(monitor_id: &MonitorId, workspace_id: &WorkspaceId) -> Result<()> {
        let mut state = trace_lock!(WM_STATE);
        let workspace = state.get_workspace_state(workspace_id);
        Self::render_workspace(monitor_id, workspace)?;
        Ok(())
    }

    pub fn clear_state() {
        trace_lock!(WM_STATE).layouts.clear();
    }

    pub fn init_state() {
        trace_lock!(WM_STATE).recreate()
    }

    pub fn enumerate_all_windows() -> Result<()> {
        WindowEnumerator::new().for_each(|window| {
            if !Self::is_managed(&window)
                && Self::should_be_managed(window.hwnd())
                && !window.is_minimized()
            {
                log_error!(Self::add(&window));
            }
        })
    }
}
