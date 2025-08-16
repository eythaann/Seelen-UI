use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::{rect::Rect, state::WmNodeKind};
use std::sync::Arc;

use crate::{
    error::Result,
    modules::input::Mouse,
    trace_lock,
    virtual_desktops::{events::VirtualDesktopEvent, get_vd_manager},
    widgets::window_manager::node_ext::WmNodeExt,
    windows_api::{
        monitor::Monitor,
        window::{event::WinEvent, Window},
    },
};

use super::{cli::Axis, state::WM_STATE, WindowManagerV2};

lazy_static! {
    static ref SystemMoveSizeStartRect: Arc<Mutex<Rect>> = Arc::new(Mutex::new(Rect::default()));
    static ref SystemMoveSizeStartMonitor: Arc<Mutex<Monitor>> =
        Arc::new(Mutex::new(Monitor::from(0)));
}

impl WindowManagerV2 {
    pub fn process_vd_event(event: &VirtualDesktopEvent) -> Result<()> {
        match event {
            VirtualDesktopEvent::DesktopChanged { monitor, workspace } => {
                // TODO: implement
                // Self::discard_reservation()?;
                Self::workspace_changed(monitor, workspace)?;
            }
            VirtualDesktopEvent::WindowAdded { window, desktop: _ } => {
                let window = &Window::from(*window);
                if !Self::is_managed(window) && Self::should_be_managed(window.hwnd()) {
                    Self::add(window)?;
                }
            }
            VirtualDesktopEvent::WindowMoved { window, .. } => {
                let window = &Window::from(*window);
                if Self::is_managed(window) {
                    Self::remove(window)?;
                    Self::add(window)?;
                }
            }
            VirtualDesktopEvent::WindowRemoved { window } => {
                let window = &Window::from(*window);
                if Self::is_managed(window) {
                    Self::remove(window)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn system_move_size_end(window: &Window) -> Result<()> {
        if !Self::is_tiled(window) {
            return Ok(());
        }

        if *trace_lock!(SystemMoveSizeStartMonitor) != window.monitor() {
            log::trace!("window moved of monitor");
            Self::remove(window)?;
            Self::add(window)?;
            return Ok(());
        }

        let initial_rect = trace_lock!(SystemMoveSizeStartRect);
        let end_rect = window.inner_rect()?;

        let initial_width = (initial_rect.right - initial_rect.left) as f32;
        let initial_height = (initial_rect.bottom - initial_rect.top) as f32;

        let new_width = (end_rect.right - end_rect.left) as f32;
        let new_height = (end_rect.bottom - end_rect.top) as f32;

        let mut state = trace_lock!(WM_STATE);
        let monitor_id = window.get_cached_data().monitor;

        // not resized only dragged/moved
        if initial_width == new_width && initial_height == new_height {
            if let Some(workspace) = state.get_workspace_state_for_window(window) {
                if let Some(node) =
                    workspace.get_node_at_point(&Mouse::get_cursor_pos()?, &[window.address()])
                {
                    if let Some(face) = node.face() {
                        workspace.swap_nodes_containing_window(window, &face)?;
                    }
                }
                Self::render_workspace(&monitor_id, workspace)?;
            }
            return Ok(());
        }

        if initial_width != new_width {
            let percentage_diff = (new_width - initial_width) / initial_width * 100.0;
            let axis = if end_rect.left == initial_rect.left {
                Axis::Right
            } else {
                Axis::Left
            };
            state.update_size(window, axis, percentage_diff, true)?;
            log::trace!("window width changed by: {percentage_diff}%");
        }

        if initial_height != new_height {
            let percentage_diff = (new_height - initial_height) / initial_height * 100.0;
            let axis = if end_rect.top == initial_rect.top {
                Axis::Bottom
            } else {
                Axis::Top
            };
            state.update_size(window, axis, percentage_diff, true)?;
            log::trace!("window height changed by: {percentage_diff}%");
        }

        let current_workspace = get_vd_manager()
            .get_active_workspace_id(&monitor_id)
            .clone();
        Self::render_workspace(&monitor_id, state.get_workspace_state(&current_workspace))?;
        Self::force_retiling()?;
        Ok(())
    }

    pub fn process_win_event(event: WinEvent, window: &Window) -> Result<()> {
        match event {
            WinEvent::SystemMoveSizeStart => {
                if Self::is_tiled(window) {
                    *trace_lock!(SystemMoveSizeStartRect) = window.inner_rect()?;
                    *trace_lock!(SystemMoveSizeStartMonitor) = window.monitor();
                }
            }
            WinEvent::SystemMoveSizeEnd => {
                Self::system_move_size_end(window)?;
            }
            WinEvent::SystemMinimizeStart => {
                let mut should_remove = false;
                {
                    let mut state = trace_lock!(WM_STATE);
                    if let Some(workspace) = state.get_workspace_state_for_window(window) {
                        if let Some(node) = workspace.layout.structure.leaf_containing(window) {
                            should_remove = node.kind != WmNodeKind::Stack;
                        }
                    }
                };
                if should_remove {
                    Self::remove(window)?;
                }
            }
            WinEvent::SystemMinimizeEnd => {
                if !Self::is_managed(window) && Self::should_be_managed(window.hwnd()) {
                    Self::add(window)?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}
