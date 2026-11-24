use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use seelen_core::{
    handlers::SeelenEvent,
    rect::Rect,
    resource::PluginId,
    state::{
        NodeId, TwmGlobalRuntimeTree, TwmRuntimeTree, WindowLocation, WorkspaceId,
        twm::{TwmNodeKind, TwmPlugin, TwmReservation, TwmStackPolicy},
    },
    system_state::{WindowHandle, WindowIdentity, ZOrder},
};

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    event_manager,
    platform::hook::HookManager,
    platform::monitor::Monitor,
    platform::window::{Window, WindowState},
    state::FULL_STATE,
    system_modules::apps::UserAppsManager,
    utils::lock_free::TracedMutex,
    virtual_desktops::VdManager,
    widgets::window_manager::{
        WindowManagerV2,
        cli::{Axis, Direction, StepWay},
        handler::{set_app_window_position, set_app_windows_positions},
        storage::{self, TwmStoredState, TwmStoredWorkspace},
    },
};

pub static MINIMIZED_BY_STACK: LazyLock<scc::HashSet<WindowHandle>> =
    LazyLock::new(scc::HashSet::new);

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
    layout_cache: HashMap<WorkspaceId, PluginId>,
}

#[derive(Debug, Clone)]
pub enum TwmStateEvent {
    Changed,
}

event_manager!(TwmState, TwmStateEvent);

impl TwmState {
    fn initialize(&mut self) {
        // layout of the last app run in this same user session
        let mut stored = storage::load_stored().unwrap_or_else(|e| {
            log::debug!("No stored tiling layout to restore: {e}");
            TwmStoredState::default()
        });
        let settings = FULL_STATE.load();

        let vd = VdManager::instance();
        vd.monitors.for_each(|(_, monitor)| {
            let active_workspace_id = monitor.active_workspace_id().clone();
            for row in monitor.workspaces.rows() {
                for workspace in row {
                    // Windows on inactive workspaces are minimized by the virtual desktop
                    // manager to hide them and must keep their slot; on the active workspace
                    // a minimized window was minimized by the user and must not get one,
                    // otherwise the layout shows an empty tile until the next re-render.
                    let should_be_managed = if workspace.id == active_workspace_id {
                        WindowManagerV2::should_be_managed
                    } else {
                        WindowManagerV2::should_be_managed_ignoring_minimized
                    };
                    // workspaces removed while the app was down are never visited, so dropped.
                    let is_monocle = stored.monocle.get(&workspace.id).copied().unwrap_or(false);
                    let layout_id = settings.get_wm_layout_id(&workspace.id);
                    let restored = stored
                        .workspaces
                        .remove(&workspace.id)
                        // the layout changed while the app was down
                        .filter(|s| is_monocle || s.layout == layout_id)
                        .and_then(|s| Self::restore_tree(s, &workspace.windows, should_be_managed));

                    let mut tree = match restored {
                        Some(tree) => {
                            if is_monocle {
                                self.monocle.insert(workspace.id.clone(), true);
                            }
                            tree
                        }
                        None => Self::create_tree(&workspace.id),
                    };

                    for id in &workspace.windows {
                        if tree.contains(id.hwnd) {
                            continue;
                        }
                        let window = Window::from(id.hwnd);
                        if should_be_managed(&window) {
                            let residual = tree.add_to_tiled(*id);
                            for w in residual {
                                tree.add_to_floating(w.hwnd);
                            }
                        }
                    }
                    self.state.workspaces.insert(workspace.id.clone(), tree);
                }
            }
        });
        self.state.paused = stored.paused;

        VdManager::subscribe(|event| {
            WindowManagerV2::process_vd_event(event).log_error();
        });

        HookManager::subscribe(|(event, origin)| {
            WindowManagerV2::process_window_event(event, origin).log_error()
        });

        self.recompute_paused_by_monitor();
        UserAppsManager::subscribe(|_event| {
            let mut state = WM_STATE.lock();
            if state.recompute_paused_by_monitor() {
                TwmState::send(TwmStateEvent::Changed);
            }
        });

        for workspace_id in self.state.workspaces.keys() {
            self.layout_cache.insert(
                workspace_id.clone(),
                settings.get_wm_layout_id(workspace_id),
            );
        }

        TwmState::subscribe(|_event| storage::request_save());
        // persist the sanitized state
        storage::request_save();
    }

    /// Restores a tree stored by a previous app run, keeping only the windows that are still
    /// alive (same identity, so handles reused by the OS are discarded) and in its workspace.
    fn restore_tree(
        stored: TwmStoredWorkspace,
        workspace_windows: &[WindowIdentity],
        should_be_managed: fn(&Window) -> bool,
    ) -> Option<TwmRuntimeTree> {
        let mut tree = stored.tree;
        if !tree.sanitize_structure() {
            return None;
        }
        tree.restore_runtime_indexes();

        let residual = tree.retain_windows(|id| {
            workspace_windows.contains(id) && should_be_managed(&Window::from(id.hwnd))
        });
        for w in residual {
            tree.add_to_floating(w.hwnd);
        }

        for id in stored.floating {
            if workspace_windows.contains(&id) && !tree.contains(id.hwnd) {
                tree.add_to_floating(id.hwnd);
            }
        }
        Some(tree)
    }

    /// Snapshot of the state to be persisted, see [`storage`].
    pub fn to_stored(&self) -> TwmStoredState {
        let settings = FULL_STATE.load();
        let workspaces = self
            .state
            .workspaces
            .iter()
            .map(|(id, tree)| {
                let floating = tree
                    .window_map
                    .iter()
                    .filter(|(_, location)| matches!(location, WindowLocation::Floating))
                    .map(|(&hwnd, _)| Window::from(hwnd).id())
                    .collect();
                let layout = self
                    .layout_cache
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| settings.get_wm_layout_id(id));
                let stored = TwmStoredWorkspace {
                    layout,
                    tree: tree.clone(),
                    floating,
                };
                (id.clone(), stored)
            })
            .collect();

        TwmStoredState {
            workspaces,
            monocle: self
                .monocle
                .iter()
                .filter(|(id, enabled)| **enabled && self.state.workspaces.contains_key(*id))
                .map(|(id, enabled)| (id.clone(), *enabled))
                .collect(),
            paused: self.state.paused,
        }
    }

    pub fn on_settings_changed(&mut self) {
        let settings = FULL_STATE.load();
        let workspace_ids: Vec<WorkspaceId> = self.state.workspaces.keys().cloned().collect();
        for workspace_id in workspace_ids {
            let new_id = settings.get_wm_layout_id(&workspace_id);
            if self.layout_cache.get(&workspace_id) == Some(&new_id) {
                continue;
            }

            self.layout_cache.insert(workspace_id.clone(), new_id);
            if self.monocle.get(&workspace_id).copied().unwrap_or(false) {
                continue;
            }

            let new_tree = Self::create_tree(&workspace_id);
            self.change_layout(&workspace_id, new_tree);
        }
    }

    fn create_tree(workspace_id: &WorkspaceId) -> TwmRuntimeTree {
        let settings = FULL_STATE.load();
        let layout = settings.get_wm_layout(workspace_id);
        TwmRuntimeTree::from_plugin(&layout)
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
                        Window::from(w.hwnd)
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

        let identity = window.id();
        node.windows.push(identity);
        node.active_window = Some(identity);
        tree.window_map.insert(
            window.handle(),
            WindowLocation::Tiled(found_node_id, std::time::SystemTime::now()),
        );

        true
    }

    pub fn is_managed(&self, window: &Window) -> bool {
        self.state.contains(window.handle())
    }

    pub fn is_tiled(&self, window: &Window) -> bool {
        self.state.is_tiled(window.handle())
    }

    pub fn add_to_floating(&mut self, window: &Window, workspace_id: &WorkspaceId) {
        let tree = self.get_or_insert_tree_mut(workspace_id);
        tree.add_to_floating(window.handle());
    }

    pub fn reserve(&mut self, side: TwmReservation) {
        let foreground = Window::get_foregrounded();
        if !self.is_tiled(&foreground) {
            return;
        }

        let hwnd = foreground.handle();
        for (ws_id, tree) in &self.state.workspaces {
            if let Some(node_id) = tree.node_of_window(hwnd) {
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
                twm_set_rect_to_float_initial_size(window, &window.monitor()).log_error();
            }
            TwmReservation::Stack => {
                let tree = self.get_or_insert_tree_mut(workspace_id);
                if let Some(node) = tree.nodes.get_mut(&node_id) {
                    if node.kind != TwmNodeKind::Stack {
                        node.kind = TwmNodeKind::Stack;
                        node.stack_policy = TwmStackPolicy::Manual;
                    }
                    let identity = window.id();
                    node.windows.push(identity);
                    node.active_window = Some(identity);
                    tree.window_map.insert(
                        window.handle(),
                        WindowLocation::Tiled(node_id, std::time::SystemTime::now()),
                    );
                } else {
                    let residual = tree.add_to_tiled(window.id());
                    for w in residual {
                        tree.add_to_floating(w.hwnd);
                    }
                }
            }
            side => {
                let tree = self.get_or_insert_tree_mut(workspace_id);
                let identity = window.id();
                let placed = tree.split_node_for_reservation(node_id, side, identity);
                if !placed {
                    let residual = tree.add_to_tiled(identity);
                    for w in residual {
                        tree.add_to_floating(w.hwnd);
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

        let residual = tree.add_to_tiled(window.id());
        for w in residual {
            tree.add_to_floating(w.hwnd);
            let residual_window = Window::from(w.hwnd);
            twm_set_rect_to_float_initial_size(&residual_window, &residual_window.monitor())
                .log_error();
        }
    }

    /// The window could be already destroyed, so it is only used by handle here.
    pub fn remove(&mut self, window: &Window) {
        let hwnd = window.handle();
        for tree in self.state.workspaces.values_mut() {
            if tree.contains(hwnd) {
                let residual = tree.remove_window(hwnd);
                for w in residual {
                    tree.add_to_floating(w.hwnd);
                    let residual_window = Window::from(w.hwnd);
                    twm_set_rect_to_float_initial_size(
                        &residual_window,
                        &residual_window.monitor(),
                    )
                    .log_error();
                }
                break;
            }
        }
    }

    /// Drops all the state related to a destroyed workspace.
    pub fn remove_workspace(&mut self, workspace_id: &WorkspaceId) {
        self.state.workspaces.remove(workspace_id);
        self.monocle.remove(workspace_id);
        self.layout_cache.remove(workspace_id);
        if self
            .pending_reservation
            .as_ref()
            .is_some_and(|r| &r.workspace_id == workspace_id)
        {
            self.cancel_reservation();
        }
    }

    pub fn get_tree_for_window_mut(
        &mut self,
        window: &Window,
    ) -> Option<(WorkspaceId, &mut TwmRuntimeTree)> {
        let hwnd = window.handle();
        let workspace_id = self
            .state
            .workspaces
            .iter()
            .find(|(_, t)| t.contains(hwnd))
            .map(|(id, _)| id.clone())?;
        let tree = self.state.workspaces.get_mut(&workspace_id)?;
        Some((workspace_id, tree))
    }

    /// Updates cached `node.rect` for a window when layout positions are applied.
    /// Uses `try_lock` internally; call this from non-lock-holding contexts.
    pub fn set_cached_node_rect(&mut self, hwnd: WindowHandle, rect: Rect) {
        for tree in self.state.workspaces.values_mut() {
            if let Some(node_id) = tree.node_of_window(hwnd)
                && let Some(node) = tree.nodes.get_mut(&node_id)
            {
                node.rect = Some(rect);
                return;
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
        let hwnd = window.handle();

        let workspace_id = self
            .state
            .workspaces
            .iter()
            .find(|(_, t)| t.contains(hwnd))
            .map(|(id, _)| id.clone());

        let Some(workspace_id) = workspace_id else {
            return Ok(());
        };

        let tree = self.state.workspaces.get(&workspace_id).unwrap();

        if tree.is_floating(hwnd) {
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
            positions.insert(hwnd, rect);
            return set_app_windows_positions(positions);
        }

        let Some(leaf_node_id) = tree.node_of_window(hwnd) else {
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
                        .filter(|&(idx, &c)| {
                            let correct_side = match axis {
                                Axis::Horizontal | Axis::Vertical => true,
                                Axis::Left | Axis::Top => idx < child_idx,
                                Axis::Right | Axis::Bottom => idx > child_idx,
                            };
                            idx != child_idx && correct_side && tree.has_any_windows(c)
                        })
                        .map(|(_, &c)| c)
                        .collect();

                    if sibling_ids.is_empty() {
                        log::warn!(
                            "Can't change size at {axis:?} if there are no other windows on that side"
                        );
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
                new_layout.add_to_floating(r.hwnd);
            }
        }

        *old = new_layout;
        Self::send(TwmStateEvent::Changed);
    }

    pub fn toggle_paused(&mut self) {
        self.state.paused = !self.state.paused;
        Self::send(TwmStateEvent::Changed);
    }

    /// Recomputes `paused_by_monitor` from the live interactable-windows list.
    /// Returns `true` if the computed value changed.
    pub fn recompute_paused_by_monitor(&mut self) -> bool {
        let mut computed: HashMap<seelen_core::system_state::MonitorId, bool> = HashMap::new();
        for w in UserAppsManager::instance().interactable_windows.to_vec() {
            let flag = (w.is_zoomed || w.is_fullscreen) && !w.is_iconic;
            let entry = computed.entry(w.monitor.clone()).or_insert(false);
            *entry = *entry || flag;
        }

        if computed == self.state.paused_by_monitor {
            return false;
        }
        self.state.paused_by_monitor = computed;
        true
    }

    pub fn toggle_monocle(&mut self, workspace_id: &WorkspaceId) {
        let is_monocle = self.monocle.entry(workspace_id.clone()).or_insert(false);
        *is_monocle = !*is_monocle;

        let layout = if *is_monocle {
            TwmRuntimeTree::from_plugin(&TwmPlugin::monocle())
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
        log::trace!("swapping windows ({:x}, {:x})", a.handle(), b.handle());
        let tree = self
            .state
            .workspaces
            .get_mut(workspace_id)
            .ok_or("Workspace not found")?;
        tree.swap_nodes_by_windows(a.handle(), b.handle());
        Self::send(TwmStateEvent::Changed);
        Ok(())
    }

    pub fn get_nearest_tiled_window_to_rect(
        &self,
        rect: &Rect,
        workspace_id: &WorkspaceId,
    ) -> Option<WindowHandle> {
        let tree = self.state.workspaces.get(workspace_id)?;
        let node_id = tree.get_nearest_leaf_to_rect(rect)?;
        tree.face_of_node(node_id).map(|face| face.hwnd)
    }

    pub fn get_nearest_tiled_window_at_side(
        &self,
        source_id: WindowHandle,
        source_rect: &Rect,
        workspace_id: &WorkspaceId,
        side: Direction,
    ) -> Option<WindowHandle> {
        fn horizontal_overlap(a: &Rect, b: &Rect) -> i32 {
            let left = a.left.max(b.left);
            let right = a.right.min(b.right);

            (right - left).max(0)
        }

        fn vertical_overlap(a: &Rect, b: &Rect) -> i32 {
            let top = a.top.max(b.top);
            let bottom = a.bottom.min(b.bottom);

            (bottom - top).max(0)
        }

        fn perpendicular_x_distance(a: &Rect, b: &Rect) -> i32 {
            if a.right < b.left {
                b.left - a.right
            } else if b.right < a.left {
                a.left - b.right
            } else {
                0
            }
        }

        fn perpendicular_y_distance(a: &Rect, b: &Rect) -> i32 {
            if a.bottom < b.top {
                b.top - a.bottom
            } else if b.bottom < a.top {
                a.top - b.bottom
            } else {
                0
            }
        }

        let tree = self.state.workspaces.get(workspace_id)?;
        tree.iter()
            .filter(|n| matches!(n.kind, TwmNodeKind::Leaf | TwmNodeKind::Stack))
            .filter_map(|n| {
                let rect = n.rect.as_ref()?;

                let (primary_distance, perpendicular_distance, overlap) = match side {
                    Direction::Right => {
                        if rect.left < source_rect.right {
                            return None;
                        }

                        (
                            rect.left - source_rect.right,
                            perpendicular_y_distance(source_rect, rect),
                            vertical_overlap(source_rect, rect),
                        )
                    }

                    Direction::Left => {
                        if rect.right > source_rect.left {
                            return None;
                        }

                        (
                            source_rect.left - rect.right,
                            perpendicular_y_distance(source_rect, rect),
                            vertical_overlap(source_rect, rect),
                        )
                    }

                    Direction::Down => {
                        if rect.top < source_rect.bottom {
                            return None;
                        }

                        (
                            rect.top - source_rect.bottom,
                            perpendicular_x_distance(source_rect, rect),
                            horizontal_overlap(source_rect, rect),
                        )
                    }

                    Direction::Up => {
                        if rect.bottom > source_rect.top {
                            return None;
                        }

                        (
                            source_rect.top - rect.bottom,
                            perpendicular_x_distance(source_rect, rect),
                            horizontal_overlap(source_rect, rect),
                        )
                    }
                };

                let perpendicular_penalty = perpendicular_distance * perpendicular_distance;
                let overlap_bonus = overlap * 100;
                let score = primary_distance * 1000 + perpendicular_penalty - overlap_bonus;

                Some((n.id, score))
            })
            .min_by_key(|&(_, score)| score)
            .and_then(|(node_id, _)| tree.face_of_node(node_id))
            .map(|face| face.hwnd)
            .filter(|&face| face != source_id)
    }

    pub fn restore_stacks(&self) {
        let mut active_ids = std::collections::HashSet::new();
        VdManager::instance().monitors.for_each(|(_, monitor)| {
            active_ids.insert(monitor.active_workspace_id().clone());
        });

        for (workspace_id, tree) in &self.state.workspaces {
            if !active_ids.contains(workspace_id) {
                continue;
            }

            for node in tree {
                if !matches!(node.kind, TwmNodeKind::Leaf | TwmNodeKind::Stack) {
                    continue;
                }

                if let Some(active) = node.active_window {
                    MINIMIZED_BY_STACK.remove_sync(&active.hwnd);
                    let window = Window::from(active.hwnd);
                    // only sanitizes visibility, never activates: focus is the caller's job
                    // (see `reveal_stack_window`).
                    if window.is_minimized() {
                        window.set_state(WindowState::ShownNoActivate).log_error();
                        // a restored window can be drawn above the focused one even without
                        // activation, so keep the focused managed window on top.
                        let foreground = Window::get_foregrounded();
                        if self.is_managed(&foreground) {
                            foreground.set_z_order(ZOrder::Top).log_error();
                        }
                    }

                    for w in &node.windows {
                        if *w != active {
                            let _ = MINIMIZED_BY_STACK.insert_sync(w.hwnd);
                            Window::from(w.hwnd)
                                .set_state(WindowState::ForceMinimized)
                                .log_error();
                        }
                    }
                }
            }
        }
    }

    pub fn set_stack_active_window(&mut self, window: &Window) -> Result<()> {
        let hwnd = window.handle();
        let Some((_ws_id, tree)) = self.get_tree_for_window_mut(window) else {
            return Ok(());
        };
        let Some(node_id) = tree.node_of_window(hwnd) else {
            return Ok(());
        };
        let node = tree.nodes.get_mut(&node_id).ok_or("Node not found")?;

        // Already active: also covers the `WindowUnminimizedByUser` echo of `restore_stacks`
        // restoring it, which must not steal the focus.
        if node.kind != TwmNodeKind::Stack || node.active_window.is_some_and(|w| w.hwnd == hwnd) {
            return Ok(());
        }

        let identity = node
            .windows
            .iter()
            .find(|w| w.hwnd == hwnd)
            .copied()
            .ok_or("Window not in its node")?;
        node.active_window = Some(identity);
        Self::reveal_stack_window(window);
        Self::send(TwmStateEvent::Changed);
        Ok(())
    }

    pub fn cycle_stack(&mut self, window: &Window, way: StepWay) -> Result<()> {
        let hwnd = window.handle();
        let Some((_ws_id, tree)) = self.get_tree_for_window_mut(window) else {
            return Ok(());
        };
        let Some(node_id) = tree.node_of_window(hwnd) else {
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
        let next = node.windows[next_idx];
        node.active_window = Some(next);

        Self::reveal_stack_window(&Window::from(next.hwnd));
        Self::send(TwmStateEvent::Changed);
        Ok(())
    }

    /// Restores and focuses the new active window of a stack. The siblings are minimized
    /// afterwards by [`Self::restore_stacks`] on the `Changed` event.
    fn reveal_stack_window(window: &Window) {
        MINIMIZED_BY_STACK.remove_sync(&window.handle());
        window.unminimize().log_error();
        window.focus().log_error();
    }
}

// keep in sync with the bounds enforced by the settings GUI (380p - 720p)
const FLOATING_MIN_WIDTH: f64 = 676.0;
const FLOATING_MAX_WIDTH: f64 = 1280.0;
const FLOATING_MIN_HEIGHT: f64 = 380.0;
const FLOATING_MAX_HEIGHT: f64 = 720.0;

pub fn twm_set_rect_to_float_initial_size(window: &Window, monitor: &Monitor) -> Result<()> {
    let guard = FULL_STATE.load();
    if !guard.settings.by_widget.wm.enabled {
        return Ok(());
    }
    let config = &guard.settings.by_widget.wm.floating;
    let width = config.width.clamp(FLOATING_MIN_WIDTH, FLOATING_MAX_WIDTH);
    let height = config
        .height
        .clamp(FLOATING_MIN_HEIGHT, FLOATING_MAX_HEIGHT);

    let monitor_dpi = monitor.scale_factor()?;
    let monitor_rect = monitor.rect()?;
    let monitor_width = monitor_rect.right - monitor_rect.left;
    let monitor_height = monitor_rect.bottom - monitor_rect.top;

    let window_width = (width * monitor_dpi) as i32;
    let window_height = (height * monitor_dpi) as i32;

    let x = monitor_rect.left + (monitor_width - window_width) / 2;
    let y = monitor_rect.top + (monitor_height - window_height) / 2;

    set_app_window_position(
        window,
        Rect {
            left: x,
            top: y,
            right: x + window_width,
            bottom: y + window_height,
        },
    )
}
