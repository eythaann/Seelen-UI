use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use itertools::Itertools;
use parking_lot::Mutex;
use seelen_core::{
    rect::Rect,
    state::{WindowManagerLayout, WmNode, WmNodeKind, WorkspaceId},
};

use crate::{
    error::Result,
    log_error,
    modules::input::domain::Point,
    state::application::FULL_STATE,
    virtual_desktops::get_vd_manager,
    widgets::window_manager::{
        handler::{schedule_window_position, set_app_windows_positions},
        instance::WindowManagerV2,
        node_ext::WmNodeExt,
    },
    windows_api::window::Window,
};

use super::cli::Axis;

pub static WM_STATE: LazyLock<Arc<Mutex<WmState>>> = LazyLock::new(|| {
    Arc::new(Mutex::new({
        let mut state = WmState::default();
        state.recreate();
        state
    }))
});

#[derive(Debug, Default)]
pub struct WmState {
    pub layouts: HashMap<WorkspaceId, WmWorkspaceState>,
}

impl WmState {
    /// will enumarate all monitors and workspaces
    pub fn recreate(&mut self) {
        let vd = get_vd_manager();
        for workspace in vd.iter_workspaces() {
            let mut w_state = WmWorkspaceState::new(&workspace.id);
            for w in &workspace.windows {
                let window = Window::from(*w);
                if WindowManagerV2::should_be_managed(window.hwnd()) {
                    w_state.add_to_tiles(&window);
                }
            }
            self.layouts.insert(workspace.id.clone(), w_state);
        }
    }

    pub fn is_managed(&self, window: &Window) -> bool {
        self.layouts.values().any(|w| w.is_managed(window))
    }

    pub fn is_tiled(&self, window: &Window) -> bool {
        self.layouts.values().any(|w| w.is_tiled(window))
    }

    pub fn get_workspace_state(&mut self, workspace: &WorkspaceId) -> &mut WmWorkspaceState {
        self.layouts
            .entry(workspace.clone())
            .or_insert_with(|| WmWorkspaceState::new(workspace))
    }

    pub fn get_workspace_state_for_window(
        &mut self,
        window: &Window,
    ) -> Option<&mut WmWorkspaceState> {
        let window_workspace = get_vd_manager()
            .workspace_containing_window(&window.address())
            .map(|w| w.id.clone())?;
        Some(self.get_workspace_state(&window_workspace))
    }

    /// # Parameters
    ///
    /// - `window`: A reference to the window whose size is being updated.
    /// - `axis`: The axis along which the size update will occur (horizontal or vertical).
    /// - `percentage`: The percentage by which the window size will be updated. Can be positive or negative.
    /// - `relative`: Determines how the percentage is interpreted. If `true`, the percentage is relative to
    ///   the current window size. If `false`, it's relative to the total workspace size.
    ///
    pub fn update_size(
        &self,
        window: &Window,
        axis: Axis,
        percentage: f32,
        relative: bool,
    ) -> Result<()> {
        let monitor_id = window.monitor_id();
        let current_workspace = get_vd_manager()
            .get_active_workspace_id(&monitor_id)
            .clone();

        let Some(w) = self.layouts.get(&current_workspace) else {
            return Ok(());
        };

        if w.layout.floating_windows.contains(&window.address()) {
            let mut rect = window.inner_rect()?;
            match axis {
                Axis::Horizontal | Axis::Left | Axis::Right => {
                    let width = (rect.right - rect.left) as f32;
                    let center_x = rect.left + (width / 2.0) as i32;
                    let new_width = (width * (1.0 + percentage / 100.0)) as i32;
                    rect.left = center_x - new_width / 2;
                    rect.right = center_x + new_width / 2;
                }
                Axis::Vertical | Axis::Top | Axis::Bottom => {
                    let height = (rect.bottom - rect.top) as f32;
                    let center_y = rect.top + (height / 2.0) as i32;
                    let new_height = (height * (1.0 + percentage / 100.0)) as i32;
                    rect.top = center_y - new_height / 2;
                    rect.bottom = center_y + new_height / 2;
                }
            };

            let mut positions = HashMap::new();
            positions.insert(window.address(), rect);
            return set_app_windows_positions(positions);
        }

        let trace = w.trace(window);
        if trace.is_empty() {
            return Err("Trying to change size of an unmanaged window".into());
        }

        let parent_to_search = match axis {
            Axis::Horizontal | Axis::Left | Axis::Right => WmNodeKind::Horizontal,
            Axis::Vertical | Axis::Top | Axis::Bottom => WmNodeKind::Vertical,
        };

        let first_matched_parent = trace.iter().rev().find(|n| {
            n.kind == parent_to_search && n.children.iter().filter(|n| !n.is_empty()).count() >= 2
        });

        let Some(first_matched_parent) = first_matched_parent else {
            log::warn!("Can't change size if the window is alone on axis");
            return Ok(());
        };

        let (node_of_window_idx, node_of_window) = first_matched_parent
            .children
            .iter()
            .find_position(|n| n.contains(window))
            .expect("The algorithm at the top of this function is wrong / broken");

        let siblings = first_matched_parent
            .children
            .iter()
            .enumerate()
            .filter(|(idx, n)| {
                *idx != node_of_window_idx
                    && match axis {
                        Axis::Horizontal | Axis::Vertical => true,
                        Axis::Left | Axis::Top => *idx < node_of_window_idx,
                        Axis::Bottom | Axis::Right => *idx > node_of_window_idx,
                    }
                    && !n.is_empty()
            })
            .collect_vec();

        if siblings.is_empty() {
            log::warn!("Can't change size at {axis:?} if there are no other windows on that side");
            return Ok(());
        }

        let total_pie: f32 = siblings.iter().map(|(_, n)| n.grow_factor.get()).sum();
        let window_portion = node_of_window.grow_factor.get();

        let to_grow = if relative { window_portion } else { total_pie } * percentage / 100f32;
        let to_reduce = to_grow / siblings.len() as f32;

        node_of_window.grow_factor.set(window_portion + to_grow);
        for (_, n) in siblings {
            n.grow_factor.set(n.grow_factor.get() - to_reduce);
        }

        Ok(())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct WmWorkspaceState {
    pub id: WorkspaceId,
    pub layout: WindowManagerLayout,
    pub monocle: bool,
}

impl WmWorkspaceState {
    pub fn new(workspace_id: &WorkspaceId) -> Self {
        let settings = FULL_STATE.load();
        let layout = settings.get_wm_layout(workspace_id);
        Self {
            id: workspace_id.clone(),
            layout,
            monocle: false,
        }
    }

    pub fn set_rect_to_float_initial_size(window: &Window) -> Result<()> {
        let guard = FULL_STATE.load();
        let config = &guard.settings.by_widget.wm.floating;

        let monitor = window.monitor();
        let monitor_dpi = monitor.scale_factor()?;
        let monitor_rect = monitor.rect()?;
        let monitor_width = monitor_rect.right - monitor_rect.left;
        let monitor_height = monitor_rect.bottom - monitor_rect.top;

        let window_width = (config.width * monitor_dpi) as i32;
        let window_height = (config.height * monitor_dpi) as i32;

        let x = monitor_rect.left + (monitor_width - window_width) / 2;
        let y = monitor_rect.top + (monitor_height - window_height) / 2;

        schedule_window_position(
            window.address(),
            Rect {
                left: x,
                top: y,
                right: x + window_width,
                bottom: y + window_height,
            },
        );
        Ok(())
    }

    pub fn add_to_floats(&mut self, window: &Window) -> Result<()> {
        let window_id = window.address();
        if self.is_floating(&window_id) {
            return Ok(());
        }

        log::trace!("floating window ({window_id:x})");
        self.layout.floating_windows.push(window_id);
        Self::set_rect_to_float_initial_size(window)
    }

    pub fn add_to_tiles(&mut self, window: &Window) {
        if self.is_tiled(window) {
            return;
        }
        log::trace!("tiling window ({:x})", window.address());

        self.layout
            .floating_windows
            .retain(|w| w != &window.address());
        let residual = self.layout.structure.add_window(window);
        for w in residual {
            log_error!(self.add_to_floats(&Window::from(w)));
        }
    }

    pub fn unmanage(&mut self, window: &Window) {
        self.layout
            .floating_windows
            .retain(|w| w != &window.address());
        let residual = self.layout.structure.remove_window(window);
        for w in residual {
            log_error!(self.add_to_floats(&Window::from(w)));
        }
    }

    pub fn is_floating(&self, window_id: &isize) -> bool {
        self.layout.floating_windows.contains(window_id)
    }

    pub fn is_tiled(&self, window: &Window) -> bool {
        self.layout.structure.contains(window)
    }

    pub fn is_managed(&self, window: &Window) -> bool {
        self.is_floating(&window.address()) || self.is_tiled(window)
    }

    pub fn swap_nodes_containing_window(&mut self, a: &Window, b: &Window) -> Result<()> {
        if a == b {
            return Ok(());
        }
        log::trace!(
            "swapping nodes containing windows ({:x}, {:x})",
            a.address(),
            b.address()
        );
        let ptr = &mut self.layout.structure as *mut WmNode;
        let node_a = unsafe { &mut *ptr }.leaf_containing_mut(a);
        let node_b = unsafe { &mut *ptr }.leaf_containing_mut(b);

        if let (Some(node_a), Some(node_b)) = (node_a, node_b) {
            std::mem::swap(&mut node_a.kind, &mut node_b.kind);
            std::mem::swap(&mut node_a.active, &mut node_b.active);
            std::mem::swap(&mut node_a.windows, &mut node_b.windows);
        }
        Ok(())
    }

    pub fn trace(&self, window: &Window) -> Vec<&WmNode> {
        self.layout.structure.trace(window)
    }

    pub fn get_node_at_point(&mut self, point: &Point, ignore: &[isize]) -> Option<&mut WmNode> {
        self.layout
            .structure
            .get_node_at_point(point, ignore)
            .ok()?
    }

    pub fn toggle_monocle(&mut self) {
        self.monocle = !self.monocle;

        if self.monocle {
            let windows = self.layout.structure.drain();
            let active = windows.first().copied();

            self.layout.structure = WmNode {
                kind: WmNodeKind::Stack,
                active,
                windows,
                max_stack_size: None,
                ..Default::default()
            }
        } else {
            let windows = self.layout.structure.drain();
            self.layout.structure = FULL_STATE.load().get_wm_layout(&self.id).structure;
            for w in windows {
                self.add_to_tiles(&Window::from(w));
            }
        }
    }
}
