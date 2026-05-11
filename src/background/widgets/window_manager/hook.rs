use seelen_core::state::WmDragBehavior;

use crate::{
    error::Result,
    state::application::FULL_STATE,
    virtual_desktops::MINIMIZED_BY_WORKSPACES,
    widgets::window_manager::state_v2::{TwmState, TwmStateEvent, WM_STATE},
    windows_api::window::{event::WinEvent, Window},
};

use super::{cli::Axis, WindowManagerV2};

impl WindowManagerV2 {
    fn system_move_size_end(window: &Window) -> Result<()> {
        let mut state = WM_STATE.lock();
        if !state.is_tiled(window) {
            return Ok(());
        }

        let initial_rect = window.get_rect_before_dragging()?;
        let end_rect = window.inner_rect()?;

        let initial_width = (initial_rect.right - initial_rect.left) as f32;
        let initial_height = (initial_rect.bottom - initial_rect.top) as f32;
        let new_width = (end_rect.right - end_rect.left) as f32;
        let new_height = (end_rect.bottom - end_rect.top) as f32;

        // not resized — only dragged/moved
        if initial_width == new_width && initial_height == new_height {
            let drag_behavior = FULL_STATE.load().settings.by_widget.wm.drag_behavior;
            if drag_behavior == WmDragBehavior::Swap {
                let current_rect = window.inner_rect()?;
                let Some((workspace_id, _)) = state.get_tree_for_window_mut(window) else {
                    return Ok(());
                };
                if let Some(face_id) =
                    state.get_nearest_tiled_window_to_rect(&current_rect, &workspace_id)
                {
                    let face = Window::from(face_id);
                    if &face != window && current_rect.intersection(&face.inner_rect()?).is_some() {
                        state.swap_tiled_windows(window, &face, &workspace_id)?;
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

    fn synthetic_foreground_location_change(window: &Window) -> Result<()> {
        let drag_behavior = FULL_STATE.load().settings.by_widget.wm.drag_behavior;
        if drag_behavior != WmDragBehavior::Sort || !window.is_dragging() {
            return Ok(());
        }

        let mut state = WM_STATE.lock();
        if !state.is_tiled(window) {
            return Ok(());
        }

        let current_rect = window.inner_rect()?;
        let Some((workspace_id, _)) = state.get_tree_for_window_mut(window) else {
            return Ok(());
        };
        if let Some(face_id) = state.get_nearest_tiled_window_to_rect(&current_rect, &workspace_id)
        {
            let face = Window::from(face_id);
            if &face != window && current_rect.intersection(&face.inner_rect()?).is_some() {
                state.swap_tiled_windows(window, &face, &workspace_id)?;
            }
        }

        Ok(())
    }

    pub fn process_win_event(event: WinEvent, window: &Window) -> Result<()> {
        match event {
            WinEvent::SynThrottledForegroundRectChange => {
                Self::synthetic_foreground_location_change(window)?;
            }
            WinEvent::SystemMoveSizeEnd => {
                Self::system_move_size_end(window)?;
                Self::force_retiling()?;
            }
            WinEvent::SystemMinimizeStart => {
                if MINIMIZED_BY_WORKSPACES.contains(&window.address()) {
                    return Ok(());
                }
                let should_remove = {
                    let mut state = WM_STATE.lock();
                    state
                        .get_tree_for_window_mut(window)
                        .map(|(_, tree)| !tree.node_is_stack(&window.address()))
                        .unwrap_or(false)
                };
                if should_remove {
                    WM_STATE.lock().remove(window);
                    TwmState::send(TwmStateEvent::Changed);
                }
            }
            WinEvent::SystemMinimizeEnd => {
                let mut state = WM_STATE.lock();
                if !state.is_managed(window) && Self::should_be_managed(window.hwnd()) {
                    state.add_to_layout(window, &window.workspace_id()?);
                    TwmState::send(TwmStateEvent::Changed);
                }
            }
            _ => {}
        };
        Ok(())
    }
}
