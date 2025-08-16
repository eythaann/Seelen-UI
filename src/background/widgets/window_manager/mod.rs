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
    UI::WindowsAndMessaging::{WS_CAPTION, WS_EX_TOPMOST},
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
        && !window.is_cloaked()
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
        // log::trace!("rendering layout {} in monitor {monitor_id}", w.layout.structure);
        workspace.layout.structure.hide_non_active()?;
        get_app_handle().emit_to(
            Self::get_label(monitor_id),
            SeelenEvent::WMSetLayout,
            &workspace.layout.structure,
        )?;
        Ok(())
    }

    fn add(window: &Window) -> Result<()> {
        let monitor_id = window.get_cached_data().monitor;

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
        let monitor_id = window.get_cached_data().monitor;
        let mut state = trace_lock!(WM_STATE);

        let mut vd = get_vd_manager();
        let current_workspace = vd.get_active_workspace_id(&monitor_id);

        for (workspace_id, workspace) in &mut state.layouts {
            workspace.unmanage(window);
            if workspace_id == current_workspace {
                Self::render_workspace(&monitor_id, workspace)?;
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
