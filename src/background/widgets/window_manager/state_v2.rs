use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use seelen_core::{
    handlers::SeelenEvent,
    rect::Rect,
    state::{
        twm::{TwmNodeKind, TwmPlugin, TwmReservation, TwmStackPolicy},
        NodeId, TwmGlobalRuntimeTree, TwmRuntimeTree, WindowLocation, WorkspaceId,
    },
};
use windows::Win32::UI::WindowsAndMessaging::SW_FORCEMINIMIZE;

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    event_manager,
    state::application::FULL_STATE,
    utils::lock_free::TracedMutex,
    virtual_desktops::{events::VirtualDesktopEvent, SluWorkspacesManager2},
    widgets::window_manager::{
        cli::{Axis, StepWay},
        handler::{schedule_window_position, set_app_windows_positions},
        instance::WindowManagerV2,
    },
    windows_api::{monitor::Monitor, window::Window},
};

pub static WM_STATE: LazyLock<Arc<TracedMutex<TwmState>>> = LazyLock::new(|| {
    Arc::new(TracedMutex::new({
        let mut state = TwmState::default();
        state.initialize();
        state
    }))
});

pub struct PendingReservation {
    pub workspace_id: WorkspaceId,
    pub node_id: NodeId,
    pub side: TwmReservation,
}

#[derive(Default)]
pub struct TwmState {
    pub state: TwmGlobalRuntimeTree,
    pub monocle: HashMap<WorkspaceId, bool>,
    pub pending_reservation: Option<PendingReservation>,
}

#[derive(Debug, Clone)]
pub enum TwmStateEvent {
    Changed,
}

event_manager!(TwmState, TwmStateEvent);

impl TwmState {
    fn initialize(&mut self) {
        let vd = SluWorkspacesManager2::instance();
        vd.monitors.for_each(|(_, monitor)| {
            for workspace in &monitor.workspaces {
                let mut tree = Self::create_tree(&workspace.id);
                for &hwnd in &workspace.windows {
                    let window = Window::from(hwnd);
                    if WindowManagerV2::should_be_managed(window.hwnd()) {
                        let residual = tree.add_to_tiled(window.address());
                        for w in residual {
                            tree.add_to_floating(w);
                        }
                    }
                }
                self.state.workspaces.insert(workspace.id.clone(), tree);
            }
        });

        SluWorkspacesManager2::subscribe(|event| {
            WM_STATE.lock().process_vd_event(&event).log_error();
        });
    }

    fn create_tree(workspace_id: &WorkspaceId) -> TwmRuntimeTree {
        let settings = FULL_STATE.load();
        let layout = settings.get_wm_layout(workspace_id);
        TwmRuntimeTree::from_plugin(&layout)
    }

    pub fn process_vd_event(&mut self, event: &VirtualDesktopEvent) -> Result<()> {
        match event {
            VirtualDesktopEvent::DesktopChanged { .. } => {
                self.cancel_reservation();
                Self::send(TwmStateEvent::Changed);
            }
            VirtualDesktopEvent::WindowAdded { window, desktop } => {
                let window = &Window::from(*window);
                if !self.is_managed(window) && WindowManagerV2::should_be_managed(window.hwnd()) {
                    self.add_to_layout(window, desktop);
                    Self::send(TwmStateEvent::Changed);
                }
            }
            VirtualDesktopEvent::WindowMoved { window, desktop } => {
                let window = &Window::from(*window);
                if self.is_managed(window) {
                    self.remove(window);
                    self.add_to_layout(window, desktop);
                    Self::send(TwmStateEvent::Changed);
                }
            }
            VirtualDesktopEvent::WindowRemoved { window } => {
                let window = &Window::from(*window);
                if self.is_managed(window) {
                    self.remove(window);
                    Self::send(TwmStateEvent::Changed);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn get_or_insert_tree_mut(&mut self, workspace_id: &WorkspaceId) -> &mut TwmRuntimeTree {
        self.state
            .workspaces
            .entry(workspace_id.clone())
            .or_insert_with(|| Self::create_tree(workspace_id))
    }

    fn try_add_to_layout_categorized(window: &Window, tree: &mut TwmRuntimeTree) -> bool {
        if !FULL_STATE
            .load()
            .settings
            .by_widget
            .wm
            .auto_stacking_by_category
        {
            return false;
        }

        let Some(searching) = window.slu_category() else {
            return false;
        };

        let Some(found_node_id) = tree
            .iter()
            .find(|node| {
                matches!(node.kind, TwmNodeKind::Leaf | TwmNodeKind::Stack)
                    && node.windows.iter().any(|w| {
                        Window::from(*w)
                            .slu_category()
                            .is_some_and(|c| c == searching)
                    })
            })
            .map(|n| n.id)
        else {
            return false;
        };

        let node = tree.nodes.get_mut(&found_node_id).unwrap();

        if node.kind != TwmNodeKind::Stack {
            node.kind = TwmNodeKind::Stack;
            node.stack_policy = TwmStackPolicy::Manual;
        }

        node.windows.push(window.address());
        node.active_window = Some(window.address());
        tree.window_map
            .insert(window.address(), WindowLocation::Tiled(found_node_id));

        true
    }

    pub fn is_managed(&self, window: &Window) -> bool {
        let id = window.address();
        self.state.workspaces.values().any(|t| t.contains(&id))
    }

    pub fn is_tiled(&self, window: &Window) -> bool {
        let id = window.address();
        self.state.workspaces.values().any(|t| t.is_tiled(&id))
    }

    pub fn add_to_floating(&mut self, window: &Window, workspace_id: &WorkspaceId) {
        let tree = self.get_or_insert_tree_mut(workspace_id);
        tree.add_to_floating(window.address());
    }

    pub fn reserve(&mut self, side: TwmReservation) {
        let foreground = Window::get_foregrounded();
        if !self.is_tiled(&foreground) {
            return;
        }

        let window_id = foreground.address();
        for (ws_id, tree) in &self.state.workspaces {
            if let Some(node_id) = tree.node_of_window(&window_id) {
                self.pending_reservation = Some(PendingReservation {
                    workspace_id: ws_id.clone(),
                    node_id,
                    side,
                });
                emit_to_webviews(SeelenEvent::WMSetReservation, Some(side));
                return;
            }
        }
    }

    pub fn cancel_reservation(&mut self) {
        self.pending_reservation = None;
        emit_to_webviews(SeelenEvent::WMSetReservation, None::<TwmReservation>);
    }

    fn apply_reservation(
        &mut self,
        window: &Window,
        workspace_id: &WorkspaceId,
        node_id: NodeId,
        side: TwmReservation,
    ) {
        match side {
            TwmReservation::Float => {
                self.add_to_floating(window, workspace_id);
                set_rect_to_float_initial_size(window, &window.monitor()).log_error();
            }
            TwmReservation::Stack => {
                let tree = self.get_or_insert_tree_mut(workspace_id);
                if let Some(node) = tree.nodes.get_mut(&node_id) {
                    if node.kind != TwmNodeKind::Stack {
                        node.kind = TwmNodeKind::Stack;
                        node.stack_policy = TwmStackPolicy::Manual;
                    }
                    node.windows.push(window.address());
                    node.active_window = Some(window.address());
                    tree.window_map
                        .insert(window.address(), WindowLocation::Tiled(node_id));
                } else {
                    let residual = tree.add_to_tiled(window.address());
                    for w in residual {
                        tree.add_to_floating(w);
                    }
                }
            }
            side => {
                let tree = self.get_or_insert_tree_mut(workspace_id);
                let placed = tree.split_node_for_reservation(node_id, side, window.address());
                if !placed {
                    let residual = tree.add_to_tiled(window.address());
                    for w in residual {
                        tree.add_to_floating(w);
                    }
                }
            }
        }
    }

    pub fn add_to_layout(&mut self, window: &Window, workspace_id: &WorkspaceId) {
        if let Some(reservation) = self.pending_reservation.take() {
            emit_to_webviews(SeelenEvent::WMSetReservation, None::<TwmReservation>);
            if &reservation.workspace_id == workspace_id {
                self.apply_reservation(window, workspace_id, reservation.node_id, reservation.side);
                return;
            }
            // workspace mismatch: reservation discarded, fall through to normal layout
        }

        let tree = self.get_or_insert_tree_mut(workspace_id);

        if Self::try_add_to_layout_categorized(window, tree) {
            return;
        }

        let residual = tree.add_to_tiled(window.address());
        for w in residual {
            tree.add_to_floating(w);
            set_rect_to_float_initial_size(window, &window.monitor()).log_error();
        }
    }

    pub fn remove(&mut self, window: &Window) {
        let window_id = window.address();
        for tree in self.state.workspaces.values_mut() {
            if tree.contains(&window_id) {
                let residual = tree.remove_window(&window_id);
                for w in residual {
                    tree.add_to_floating(w);
                    set_rect_to_float_initial_size(window, &window.monitor()).log_error();
                }
                break;
            }
        }
    }

    pub fn get_tree_for_window_mut(
        &mut self,
        window: &Window,
    ) -> Option<(WorkspaceId, &mut TwmRuntimeTree)> {
        let window_id = window.address();
        let workspace_id = self
            .state
            .workspaces
            .iter()
            .find(|(_, t)| t.contains(&window_id))
            .map(|(id, _)| id.clone())?;
        let tree = self.state.workspaces.get_mut(&workspace_id)?;
        Some((workspace_id, tree))
    }

    /// Updates cached `node.rect` for a window when layout positions are applied.
    /// Uses `try_lock` internally; call this from non-lock-holding contexts.
    pub fn set_cached_node_rect(&mut self, window_id: isize, rect: Rect) {
        for tree in self.state.workspaces.values_mut() {
            if let Some(node_id) = tree.node_of_window(&window_id) {
                if let Some(node) = tree.nodes.get_mut(&node_id) {
                    node.rect = Some(rect);
                    return;
                }
            }
        }
    }

    pub fn update_size(
        &mut self,
        window: &Window,
        axis: Axis,
        percentage: f32,
        relative: bool,
    ) -> Result<()> {
        let window_id = window.address();

        let workspace_id = self
            .state
            .workspaces
            .iter()
            .find(|(_, t)| t.contains(&window_id))
            .map(|(id, _)| id.clone());

        let Some(workspace_id) = workspace_id else {
            return Ok(());
        };

        let tree = self.state.workspaces.get(&workspace_id).unwrap();

        if tree.is_floating(&window_id) {
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
            }
            let mut positions = HashMap::new();
            positions.insert(window_id, rect);
            return set_app_windows_positions(positions);
        }

        let Some(leaf_node_id) = tree.node_of_window(&window_id) else {
            return Err("Trying to change size of an unmanaged window".into());
        };

        let wanted_kind = match axis {
            Axis::Horizontal | Axis::Left | Axis::Right => TwmNodeKind::Horizontal,
            Axis::Vertical | Axis::Top | Axis::Bottom => TwmNodeKind::Vertical,
        };

        // Walk up via parent links to find the first ancestor of matching kind
        // with ≥2 non-empty children, then filter siblings by side direction.
        let mut current_id = leaf_node_id;
        loop {
            let Some(parent_id) = tree.nodes[&current_id].parent else {
                log::warn!("Can't change size if the window is alone on axis");
                return Ok(());
            };
            let parent = &tree.nodes[&parent_id];
            if parent.kind == wanted_kind {
                let child_idx = parent
                    .children
                    .iter()
                    .position(|&c| c == current_id)
                    .unwrap();
                let non_empty_count = parent
                    .children
                    .iter()
                    .filter(|&&c| tree.has_any_windows(c))
                    .count();
                if non_empty_count >= 2 {
                    let sibling_ids: Vec<u64> = parent
                        .children
                        .iter()
                        .enumerate()
                        .filter(|(idx, &c)| {
                            let correct_side = match axis {
                                Axis::Horizontal | Axis::Vertical => true,
                                Axis::Left | Axis::Top => *idx < child_idx,
                                Axis::Right | Axis::Bottom => *idx > child_idx,
                            };
                            *idx != child_idx && correct_side && tree.has_any_windows(c)
                        })
                        .map(|(_, &c)| c)
                        .collect();

                    if sibling_ids.is_empty() {
                        log::warn!("Can't change size at {axis:?} if there are no other windows on that side");
                        return Ok(());
                    }

                    let node_of_window_id = parent.children[child_idx];
                    let tree_mut = self.state.workspaces.get_mut(&workspace_id).unwrap();

                    let total_pie: f32 = sibling_ids
                        .iter()
                        .map(|&id| tree_mut.nodes[&id].grow_factor)
                        .sum();
                    let window_portion = tree_mut.nodes[&node_of_window_id].grow_factor;
                    let to_grow =
                        if relative { window_portion } else { total_pie } * percentage / 100.0;
                    let to_reduce = to_grow / sibling_ids.len() as f32;

                    tree_mut
                        .nodes
                        .get_mut(&node_of_window_id)
                        .unwrap()
                        .grow_factor += to_grow;
                    for sib_id in sibling_ids {
                        tree_mut.nodes.get_mut(&sib_id).unwrap().grow_factor -= to_reduce;
                    }

                    Self::send(TwmStateEvent::Changed);
                    return Ok(());
                }
            }
            current_id = parent_id;
        }
    }

    pub fn change_layout(&mut self, workspace_id: &WorkspaceId, mut new_layout: TwmRuntimeTree) {
        let Some(old) = self.state.workspaces.get_mut(workspace_id) else {
            return;
        };

        let windows = old.drain_tiled();
        for floating in old.window_map.keys() {
            new_layout.add_to_floating(*floating);
        }

        for w in windows {
            let residuals = new_layout.add_to_tiled(w);
            for r in residuals {
                new_layout.add_to_floating(r);
            }
        }

        *old = new_layout;
        Self::send(TwmStateEvent::Changed);
    }

    pub fn toggle_monocle(&mut self, workspace_id: &WorkspaceId) {
        let is_monocle = self.monocle.entry(workspace_id.clone()).or_insert(false);
        *is_monocle = !*is_monocle;

        let layout = if *is_monocle {
            TwmRuntimeTree::from_plugin(&TwmPlugin::default())
        } else {
            TwmRuntimeTree::from_plugin(&FULL_STATE.load().get_wm_layout(workspace_id))
        };
        self.change_layout(workspace_id, layout);
        Self::send(TwmStateEvent::Changed);
    }

    pub fn swap_tiled_windows(
        &mut self,
        a: &Window,
        b: &Window,
        workspace_id: &WorkspaceId,
    ) -> Result<()> {
        if a == b {
            return Ok(());
        }
        log::trace!("swapping windows ({:x}, {:x})", a.address(), b.address());
        let tree = self
            .state
            .workspaces
            .get_mut(workspace_id)
            .ok_or("Workspace not found")?;
        tree.swap_nodes_by_windows(a.address(), b.address());
        Self::send(TwmStateEvent::Changed);
        Ok(())
    }

    pub fn get_nearest_tiled_window_to_rect(
        &self,
        rect: &Rect,
        workspace_id: &WorkspaceId,
    ) -> Option<isize> {
        let tree = self.state.workspaces.get(workspace_id)?;
        let node_id = tree.get_nearest_leaf_to_rect(rect)?;
        tree.face_of_node(node_id)
    }

    pub fn restore_stacks(&self) {
        let mut active_ids = std::collections::HashSet::new();
        SluWorkspacesManager2::instance()
            .monitors
            .for_each(|(_, monitor)| {
                active_ids.insert(monitor.active_workspace_id().clone());
            });

        for (workspace_id, tree) in &self.state.workspaces {
            if !active_ids.contains(workspace_id) {
                continue;
            }

            for node in tree {
                if node.kind != TwmNodeKind::Stack {
                    continue;
                }

                if let Some(active) = node.active_window {
                    Window::from(active).unminimize().log_error();
                    for w in &node.windows {
                        if *w != active {
                            Window::from(*w).show_window(SW_FORCEMINIMIZE).log_error();
                        }
                    }
                }
            }
        }
    }

    pub fn cycle_stack(&mut self, window: &Window, way: StepWay) -> Result<()> {
        let window_id = window.address();
        let Some((_ws_id, tree)) = self.get_tree_for_window_mut(window) else {
            return Ok(());
        };
        let Some(node_id) = tree.node_of_window(&window_id) else {
            return Ok(());
        };
        let node = tree.nodes.get_mut(&node_id).ok_or("Node not found")?;

        if node.kind != TwmNodeKind::Stack {
            return Ok(());
        }

        let active = node.active_window.ok_or("No active window")?;
        let idx = node
            .windows
            .iter()
            .position(|&w| w == active)
            .ok_or("Active window not in list")?;
        let len = node.windows.len();
        let next_idx = if way == StepWay::Next {
            (idx + 1) % len
        } else {
            (idx + len - 1) % len
        };
        node.active_window = Some(node.windows[next_idx]);

        Self::send(TwmStateEvent::Changed);
        Ok(())
    }
}

pub fn set_rect_to_float_initial_size(window: &Window, monitor: &Monitor) -> Result<()> {
    let guard = FULL_STATE.load();
    let config = &guard.settings.by_widget.wm.floating;

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
