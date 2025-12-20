use seelen_core::state::WmNodeKind;

use crate::{
    error::Result,
    trace_lock,
    virtual_desktops::MINIMIZED_BY_WORKSPACES,
    widgets::window_manager::state::node_ext::WmNodeExt,
    windows_api::window::{event::WinEvent, Window},
};

use super::{cli::Axis, state::WM_STATE, WindowManagerV2};

impl WindowManagerV2 {
    fn system_move_size_end(window: &Window) -> Result<()> {
        let mut state = trace_lock!(WM_STATE);
        if !state.is_tiled(window) {
            return Ok(());
        }

        /* if *trace_lock!(SystemMoveSizeStartMonitor) != window.monitor() {
            log::trace!("window moved of monitor");
            Self::remove(window)?;
            Self::add(window)?;
            return Ok(());
        } */

        let initial_rect = window.get_rect_before_dragging()?;
        let end_rect = window.inner_rect()?;

        let initial_width = (initial_rect.right - initial_rect.left) as f32;
        let initial_height = (initial_rect.bottom - initial_rect.top) as f32;

        let new_width = (end_rect.right - end_rect.left) as f32;
        let new_height = (end_rect.bottom - end_rect.top) as f32;

        // not resized only dragged/moved
        if initial_width == new_width && initial_height == new_height {
            let Some(workspace) = state.get_workspace_state_for_window(window) else {
                return Ok(());
            };

            let current_rect = window.inner_rect()?;
            if let Some(node) = workspace.get_nearest_node_to_rect(&current_rect) {
                if let Some(face) = node.face() {
                    if &face != window
                            // don't swap if nearest is not overlapped
                            && current_rect.intersection(&face.inner_rect()?).is_some()
                    {
                        workspace.swap_nodes_containing_window(window, &face)?;
                    }
                }
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
        Ok(())
    }

    pub fn process_win_event(event: WinEvent, window: &Window) -> Result<()> {
        match event {
            WinEvent::SystemMoveSizeEnd => {
                Self::system_move_size_end(window)?;
                Self::force_retiling()?;
            }
            WinEvent::SystemMinimizeStart => {
                if MINIMIZED_BY_WORKSPACES.contains(&window.address()) {
                    return Ok(());
                }

                let mut should_remove = false;
                let mut state = trace_lock!(WM_STATE);
                if let Some(workspace) = state.get_workspace_state_for_window(window) {
                    if let Some(node) = workspace.layout.structure.leaf_containing(window) {
                        should_remove = node.kind != WmNodeKind::Stack;
                    }
                }

                if should_remove {
                    state.remove(window)?;
                }
            }
            WinEvent::SystemMinimizeEnd => {
                let mut state = trace_lock!(WM_STATE);
                if !state.is_managed(window) && Self::should_be_managed(window.hwnd()) {
                    state.add(window)?;
                }
            }
            _ => {}
        };
        Ok(())
    }
}
