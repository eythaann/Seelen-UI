use std::collections::HashMap;

use crate::{
    Rect,
    state::{
        WorkspaceId,
        twm::{
            TwmCondition, TwmConditionContext, TwmNodeKind, TwmNodeLifetime, TwmPlugin,
            TwmPluginNode, TwmReservation, TwmStackPolicy,
        },
    },
    system_state::{MonitorId, WindowHandle, WindowIdentity},
};

pub type NodeId = u64;

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct TwmGlobalRuntimeTree {
    pub workspaces: HashMap<WorkspaceId, TwmRuntimeTree>,
    pub paused: bool,
    pub paused_by_monitor: HashMap<MonitorId, bool>,
}

impl TwmGlobalRuntimeTree {
    pub fn contains(&self, hwnd: WindowHandle) -> bool {
        self.workspaces.iter().any(|(_, tree)| tree.contains(hwnd))
    }

    pub fn is_tiled(&self, hwnd: WindowHandle) -> bool {
        self.workspaces.iter().any(|(_, tree)| tree.is_tiled(hwnd))
    }

    pub fn is_floating(&self, hwnd: WindowHandle) -> bool {
        self.workspaces
            .iter()
            .any(|(_, tree)| tree.is_floating(hwnd))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct TwmRuntimeTree {
    #[serde(skip)]
    pub next_id: NodeId,
    pub root: NodeId,
    pub nodes: HashMap<NodeId, TwmRuntimeNode>,
    #[serde(skip)]
    /// Runtime only, so windows are tracked by handle.
    pub window_map: HashMap<WindowHandle, WindowLocation>,
}

#[derive(Debug, Clone)]
pub enum WindowLocation {
    Tiled(NodeId, std::time::SystemTime),
    Floating,
}

impl TwmRuntimeTree {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            root: 0,
            nodes: HashMap::new(),
            window_map: HashMap::new(),
        }
    }

    pub fn iter(&self) -> TwmTreeIter<'_> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> TwmTreeIterMut<'_> {
        self.into_iter()
    }

    pub fn generate_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn from_plugin(tree: &TwmPlugin) -> Self {
        let mut runtime = Self::new();
        if let Some(root) = &tree.structure {
            let root_id = runtime.insert_plugin_node(root, None);
            runtime.root = root_id;
        }
        runtime
    }

    fn insert_plugin_node(&mut self, node: &TwmPluginNode, parent: Option<NodeId>) -> NodeId {
        let mut runtime_node = TwmRuntimeNode::from_plugin(node);
        runtime_node.parent = parent;

        let id = self.generate_id();
        runtime_node.id = id;
        self.nodes.insert(id, runtime_node);

        let child_ids: Vec<NodeId> = node
            .children
            .iter()
            .map(|child| self.insert_plugin_node(child, Some(id)))
            .collect();

        self.nodes.get_mut(&id).unwrap().children = child_ids;
        id
    }

    pub fn contains(&self, hwnd: WindowHandle) -> bool {
        self.window_map.contains_key(&hwnd)
    }

    pub fn is_tiled(&self, hwnd: WindowHandle) -> bool {
        matches!(self.window_map.get(&hwnd), Some(WindowLocation::Tiled(..)))
    }

    pub fn is_floating(&self, hwnd: WindowHandle) -> bool {
        matches!(self.window_map.get(&hwnd), Some(WindowLocation::Floating))
    }

    pub fn reset_sizes(&mut self) {
        for node in self {
            node.rect = None;
            node.grow_factor = node.initial_grow_factor;
        }
    }

    // TODO: consider cached counters if condition eval becomes hot path
    fn get_context(&self) -> TwmConditionContext {
        let mut tiling_windows = 0;
        for window in self.window_map.values() {
            if let WindowLocation::Tiled(..) = window {
                tiling_windows += 1;
            }
        }
        TwmConditionContext {
            tiling_windows,
            is_reindexing: false,
        }
    }

    /// returns true if the window was added, false in case of overflow
    fn try_add_window(&mut self, window_id: WindowIdentity, ctx: &TwmConditionContext) -> bool {
        if let Some(node_id) = self.iter().find(|n| n.accepts_windows(ctx)).map(|n| n.id) {
            let node = self.nodes.get_mut(&node_id).unwrap();
            node.windows.push(window_id);
            node.active_window = Some(window_id);
            self.window_map.insert(
                window_id.hwnd,
                WindowLocation::Tiled(node_id, std::time::SystemTime::now()),
            );
            return true;
        }

        if let Some(node_id) = self
            .iter()
            .find(|n| n.accepts_windows_on_overflow(ctx))
            .map(|n| n.id)
        {
            let node = self.nodes.get_mut(&node_id).unwrap();
            node.windows.push(window_id);
            node.active_window = Some(window_id);
            self.window_map.insert(
                window_id.hwnd,
                WindowLocation::Tiled(node_id, std::time::SystemTime::now()),
            );
            return true;
        }

        false
    }

    pub fn drain_tiled(&mut self) -> Vec<WindowIdentity> {
        let mut drained = Vec::new();
        for node in self.iter_mut() {
            drained.append(&mut node.windows);
            node.active_window = None;
        }
        self.window_map
            .retain(|_, location| matches!(location, WindowLocation::Floating));
        drained
    }

    pub fn normalize(&mut self) {
        // Collapse Manual stacks that are now no longer needed
        for node in self.nodes.values_mut() {
            if node.kind == TwmNodeKind::Stack
                && node.stack_policy == TwmStackPolicy::Manual
                && node.windows.len() <= 1
            {
                node.kind = TwmNodeKind::Leaf;
            }
        }

        // Stabilizing loop: each step can expose new candidates for the others.
        //
        // Order within each iteration:
        //   A) Drain empty Temporal Leaf/Stack nodes → may leave H/V with fewer children.
        //   B) Remove H/V containers with 0 children → may leave parent with 1 child.
        //   C) Collapse H/V containers with exactly 1 child → parent absorbs it.
        //
        // We repeat until none of the three steps produces a change.
        loop {
            let mut changed = false;

            // A: Drain empty Temporal Leaf/Stack nodes, clean up stale parent child refs.
            let extracted: std::collections::HashSet<NodeId> = self
                .nodes
                .extract_if(|_, n| {
                    matches!(n.kind, TwmNodeKind::Leaf | TwmNodeKind::Stack)
                        && n.lifetime == TwmNodeLifetime::Temporal
                        && n.windows.is_empty()
                })
                .map(|(id, _)| id)
                .collect();
            if !extracted.is_empty() {
                changed = true;
                for node in self.nodes.values_mut() {
                    node.children.retain(|id| !extracted.contains(id));
                }
            }

            // B: Remove H/V containers that have no children left, clean up parent refs.
            let empty_containers: std::collections::HashSet<NodeId> = self
                .nodes
                .extract_if(|_, n| {
                    matches!(n.kind, TwmNodeKind::Horizontal | TwmNodeKind::Vertical)
                        && n.children.is_empty()
                })
                .map(|(id, _)| id)
                .collect();
            if !empty_containers.is_empty() {
                changed = true;
                for node in self.nodes.values_mut() {
                    node.children.retain(|id| !empty_containers.contains(id));
                }
            }

            // C: Collapse every H/V with exactly one child: the parent absorbs the child,
            // taking its content properties while keeping its own layout properties
            // (grow_factor, priority, rect) since those belong to its position in the tree.
            let to_collapse: Vec<(NodeId, NodeId)> = self
                .nodes
                .iter()
                .filter(|(_, n)| {
                    matches!(n.kind, TwmNodeKind::Horizontal | TwmNodeKind::Vertical)
                        && n.children.len() == 1
                })
                .map(|(id, n)| (*id, n.children[0]))
                .collect();
            if !to_collapse.is_empty() {
                changed = true;
                for (parent_id, child_id) in to_collapse {
                    let Some(child) = self.nodes.remove(&child_id) else {
                        continue; // already absorbed earlier in this pass
                    };

                    // Apply child properties to parent; split into two borrows so we can
                    // later mutate self.nodes for grandchildren and self.window_map.
                    let (grandchildren, windows) = {
                        let Some(parent) = self.nodes.get_mut(&parent_id) else {
                            continue;
                        };
                        parent.kind = child.kind;
                        parent.lifetime = child.lifetime;
                        parent.condition = child.condition;
                        parent.max_stack_size = child.max_stack_size;
                        parent.stack_policy = child.stack_policy;
                        parent.windows = child.windows;
                        parent.active_window = child.active_window;
                        parent.children = child.children;
                        (parent.children.clone(), parent.windows.clone())
                    };

                    for grandchild_id in grandchildren {
                        if let Some(gc) = self.nodes.get_mut(&grandchild_id) {
                            gc.parent = Some(parent_id);
                        }
                    }

                    for &window in &windows {
                        if let Some(WindowLocation::Tiled(nid, _)) =
                            self.window_map.get_mut(&window.hwnd)
                        {
                            *nid = parent_id;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }
    }

    /// reindexes windows to handle logical condition like `managed < 4` and returns residual windows
    pub fn reindex_windows(&mut self) -> Vec<WindowIdentity> {
        // Drain only Leaf nodes and single-window stacks.
        // Multi-window stacks (≥2 windows) are skipped — they stay where they are.
        let mut drained: Vec<WindowIdentity> = Vec::new();
        for node in self.iter_mut() {
            if node.kind == TwmNodeKind::Stack
                && node.stack_policy == TwmStackPolicy::Manual
                && node.windows.len() >= 2
            {
                continue;
            }
            drained.append(&mut node.windows);
            node.active_window = None;
        }

        let mut drained: Vec<(WindowIdentity, std::time::SystemTime)> = drained
            .into_iter()
            .map(|w| {
                let time = match self.window_map.remove(&w.hwnd) {
                    Some(WindowLocation::Tiled(_, t)) => t,
                    _ => std::time::SystemTime::UNIX_EPOCH,
                };
                (w, time)
            })
            .collect();
        drained.sort_by_key(|(_, t)| *t);

        let mut ctx = TwmConditionContext {
            tiling_windows: 0,
            is_reindexing: true,
        };
        let mut overflow = Vec::new();
        for (window, _) in drained {
            if self.try_add_window(window, &ctx) {
                ctx.tiling_windows += 1;
            } else {
                overflow.push(window);
            }
        }

        overflow
    }

    /// returns residual windows that should be added to floating
    pub fn add_to_tiled(&mut self, window_id: WindowIdentity) -> Vec<WindowIdentity> {
        let ctx = self.get_context();
        if !self.try_add_window(window_id, &ctx) {
            return vec![window_id];
        }
        self.reindex_windows()
    }

    pub fn add_to_floating(&mut self, hwnd: WindowHandle) {
        self.window_map.insert(hwnd, WindowLocation::Floating);
    }

    pub fn remove_window(&mut self, hwnd: WindowHandle) -> Vec<WindowIdentity> {
        let Some(location) = self.window_map.remove(&hwnd) else {
            return Vec::new();
        };

        match location {
            WindowLocation::Tiled(node_id, _) => {
                let node = self.nodes.get_mut(&node_id).unwrap();
                if node.active_window.is_some_and(|w| w.hwnd == hwnd) {
                    let next = node
                        .windows
                        .iter()
                        .position(|w| w.hwnd == hwnd)
                        .and_then(|i| {
                            node.windows
                                .get(i + 1)
                                .or_else(|| i.checked_sub(1).and_then(|p| node.windows.get(p)))
                        })
                        .copied();
                    node.windows.retain(|w| w.hwnd != hwnd);
                    node.active_window = next;
                } else {
                    node.windows.retain(|w| w.hwnd != hwnd);
                }
            }
            WindowLocation::Floating => {}
        }

        self.normalize();
        self.reindex_windows()
    }

    pub fn has_any_windows(&self, node_id: NodeId) -> bool {
        let node = &self.nodes[&node_id];
        if !node.windows.is_empty() {
            return true;
        }
        node.children.iter().any(|&c| self.has_any_windows(c))
    }

    pub fn node_of_window(&self, hwnd: WindowHandle) -> Option<NodeId> {
        match self.window_map.get(&hwnd)? {
            WindowLocation::Tiled(node_id, _) => Some(*node_id),
            WindowLocation::Floating => None,
        }
    }

    pub fn face_of_node(&self, node_id: NodeId) -> Option<WindowIdentity> {
        let node = self.nodes.get(&node_id)?;
        match node.kind {
            TwmNodeKind::Leaf | TwmNodeKind::Stack => {
                node.active_window.or_else(|| node.windows.first().copied())
            }
            TwmNodeKind::Horizontal | TwmNodeKind::Vertical => {
                let mut children = node.children.clone();
                children.sort_by_key(|id| self.nodes[id].priority);
                children.iter().find_map(|&c| self.face_of_node(c))
            }
        }
    }

    pub fn node_is_stack(&self, hwnd: WindowHandle) -> bool {
        self.node_of_window(hwnd)
            .and_then(|id| self.nodes.get(&id))
            .map(|n| n.kind == TwmNodeKind::Stack)
            .unwrap_or(false)
    }

    pub fn swap_nodes_by_windows(&mut self, a: WindowHandle, b: WindowHandle) {
        let (node_a, time_a) = match self.window_map.get(&a) {
            Some(WindowLocation::Tiled(id, time)) => (*id, *time),
            _ => return,
        };
        let (node_b, time_b) = match self.window_map.get(&b) {
            Some(WindowLocation::Tiled(id, time)) => (*id, *time),
            _ => return,
        };
        if node_a == node_b {
            return;
        }

        // SAFETY: node_a != node_b, so we're getting two distinct entries
        let ptr = &mut self.nodes as *mut HashMap<NodeId, TwmRuntimeNode>;
        let na = unsafe { &mut *ptr }.get_mut(&node_a).unwrap();
        let nb = unsafe { &mut *ptr }.get_mut(&node_b).unwrap();

        std::mem::swap(&mut na.kind, &mut nb.kind);
        std::mem::swap(&mut na.windows, &mut nb.windows);
        std::mem::swap(&mut na.active_window, &mut nb.active_window);

        let windows_a: Vec<WindowIdentity> = self.nodes[&node_a].windows.clone();
        let windows_b: Vec<WindowIdentity> = self.nodes[&node_b].windows.clone();
        for w in windows_a {
            self.window_map
                .insert(w.hwnd, WindowLocation::Tiled(node_a, time_a));
        }
        for w in windows_b {
            self.window_map
                .insert(w.hwnd, WindowLocation::Tiled(node_b, time_b));
        }
    }

    pub fn get_nearest_leaf_to_rect(&self, rect: &Rect) -> Option<NodeId> {
        let target = rect.center();
        self.iter()
            .filter(|n| matches!(n.kind, TwmNodeKind::Leaf | TwmNodeKind::Stack))
            .filter_map(|n| {
                n.rect
                    .as_ref()
                    .map(|r| (n.id, target.distance_squared(&r.center())))
            })
            .min_by_key(|&(_, d)| d)
            .map(|(id, _)| id)
    }

    /// Splits `node_id` by inserting a new intermediate Horizontal (Left/Right)
    /// or Vertical (Top/Bottom) container, then places `new_window` in a freshly
    /// created sibling Leaf on the requested side.
    /// Returns `false` if `node_id` does not exist in this tree (caller falls back).
    pub fn split_node_for_reservation(
        &mut self,
        node_id: NodeId,
        side: TwmReservation,
        new_window: WindowIdentity,
    ) -> bool {
        if !self.nodes.contains_key(&node_id) {
            return false;
        }
        let focused_node_id = node_id;
        let container_kind = match side {
            TwmReservation::Left | TwmReservation::Right => TwmNodeKind::Horizontal,
            TwmReservation::Top | TwmReservation::Bottom => TwmNodeKind::Vertical,
            _ => return false, // Stack / Float are handled by the caller
        };

        let new_leaf_id = self.generate_id();
        let container_id = self.generate_id();

        let grow_factor = self.nodes[&focused_node_id].grow_factor;
        let priority = self.nodes[&focused_node_id].priority;
        let old_parent = self.nodes[&focused_node_id].parent;

        let new_leaf = TwmRuntimeNode {
            id: new_leaf_id,
            parent: Some(container_id),
            children: vec![],
            kind: TwmNodeKind::Leaf,
            lifetime: TwmNodeLifetime::Temporal,
            priority,
            initial_grow_factor: 1.0,
            condition: None,
            max_stack_size: None,
            stack_policy: TwmStackPolicy::Manual,
            grow_factor: 1.0,
            windows: vec![new_window],
            active_window: Some(new_window),
            rect: None,
        };

        let children = match side {
            TwmReservation::Left | TwmReservation::Top => vec![new_leaf_id, focused_node_id],
            _ => vec![focused_node_id, new_leaf_id],
        };
        let container = TwmRuntimeNode {
            id: container_id,
            parent: old_parent,
            children,
            kind: container_kind,
            lifetime: TwmNodeLifetime::Temporal,
            priority,
            initial_grow_factor: grow_factor,
            condition: None,
            max_stack_size: None,
            stack_policy: TwmStackPolicy::Manual,
            grow_factor,
            windows: vec![],
            active_window: None,
            rect: None,
        };

        self.nodes.get_mut(&focused_node_id).unwrap().parent = Some(container_id);

        if let Some(parent_id) = old_parent {
            let parent = self.nodes.get_mut(&parent_id).unwrap();
            if let Some(idx) = parent.children.iter().position(|&c| c == focused_node_id) {
                parent.children[idx] = container_id;
            }
        } else {
            self.root = container_id;
        }

        self.nodes.insert(new_leaf_id, new_leaf);
        self.nodes.insert(container_id, container);
        self.window_map.insert(
            new_window.hwnd,
            WindowLocation::Tiled(new_leaf_id, std::time::SystemTime::now()),
        );
        true
    }

    /// Repairs the structure of a tree that was deserialized (e.g. restored from disk): drops the
    /// nodes unreachable from the root, fixes parent links, removes dangling or repeated children,
    /// keeps each window only once in the whole tree and fixes invalid active windows.
    ///
    /// Returns `false` if the tree has no valid root, so it can't be used.
    pub fn sanitize_structure(&mut self) -> bool {
        if !self.nodes.contains_key(&self.root) {
            return false;
        }

        let mut reachable = std::collections::HashSet::from([self.root]);
        let mut stack = vec![(self.root, None)];
        while let Some((id, parent)) = stack.pop() {
            let mut children = {
                let Some(node) = self.nodes.get_mut(&id) else {
                    continue;
                };
                node.id = id;
                node.parent = parent;

                let mut children = std::mem::take(&mut node.children);
                match node.kind {
                    TwmNodeKind::Leaf | TwmNodeKind::Stack => children.clear(),
                    TwmNodeKind::Horizontal | TwmNodeKind::Vertical => {
                        node.windows.clear();
                        node.active_window = None;
                    }
                }
                children
            };

            children.retain(|c| self.nodes.contains_key(c) && reachable.insert(*c));
            for &child in &children {
                stack.push((child, Some(id)));
            }
            self.nodes.get_mut(&id).unwrap().children = children;
        }
        self.nodes.retain(|id, _| reachable.contains(id));

        let mut seen = std::collections::HashSet::new();
        for node in self.iter_mut() {
            node.windows.retain(|w| seen.insert(w.hwnd));
            if node
                .active_window
                .is_some_and(|active| !node.windows.contains(&active))
            {
                node.active_window = node.windows.first().copied();
            }
        }
        true
    }

    /// Rebuilds the runtime only data (not serialized) of a deserialized tree: the id generator
    /// and the window map. Windows are indexed in tree order, so a later reindex keeps their order.
    /// Floating windows are not part of the nodes, they have to be added back by the caller.
    pub fn restore_runtime_indexes(&mut self) {
        self.next_id = self.nodes.keys().max().map_or(1, |max| max + 1);

        let tiled: Vec<(WindowHandle, NodeId)> = self
            .iter()
            .flat_map(|node| node.windows.iter().map(|w| (w.hwnd, node.id)))
            .collect();

        self.window_map.clear();
        for (idx, (hwnd, node_id)) in tiled.into_iter().enumerate() {
            let time =
                std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_nanos(idx as u64 + 1);
            self.window_map
                .insert(hwnd, WindowLocation::Tiled(node_id, time));
        }
    }

    /// Removes the tiled windows for which `keep` returns `false`, then normalizes the tree.
    /// Returns the residual windows that no longer fit in the layout (should go to floating).
    pub fn retain_windows(
        &mut self,
        mut keep: impl FnMut(&WindowIdentity) -> bool,
    ) -> Vec<WindowIdentity> {
        let mut removed = Vec::new();
        for node in self.nodes.values_mut() {
            node.windows.retain(|w| {
                let kept = keep(w);
                if !kept {
                    removed.push(w.hwnd);
                }
                kept
            });
            if node
                .active_window
                .is_some_and(|active| !node.windows.contains(&active))
            {
                node.active_window = node.windows.first().copied();
            }
        }

        for hwnd in removed {
            self.window_map.remove(&hwnd);
        }

        self.normalize();
        self.reindex_windows()
    }
}

impl Default for TwmRuntimeTree {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a TwmRuntimeTree {
    type Item = &'a TwmRuntimeNode;
    type IntoIter = TwmTreeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TwmTreeIter {
            tree: self,
            stack: vec![self.root],
        }
    }
}

impl<'a> IntoIterator for &'a mut TwmRuntimeTree {
    type Item = &'a mut TwmRuntimeNode;
    type IntoIter = TwmTreeIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let root = self.root;
        TwmTreeIterMut {
            tree: self,
            stack: vec![root],
        }
    }
}

pub struct TwmTreeIter<'a> {
    tree: &'a TwmRuntimeTree,
    stack: Vec<NodeId>,
}

impl<'a> Iterator for TwmTreeIter<'a> {
    type Item = &'a TwmRuntimeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node_id = self.stack.pop()?;
        let node = self.tree.nodes.get(&node_id)?;

        let mut children = node.children.clone();
        children.sort_by_key(|id| self.tree.nodes.get(id).unwrap().priority);

        for child in children.into_iter().rev() {
            self.stack.push(child);
        }

        Some(node)
    }
}

pub struct TwmTreeIterMut<'a> {
    tree: &'a mut TwmRuntimeTree,
    stack: Vec<NodeId>,
}

impl<'a> Iterator for TwmTreeIterMut<'a> {
    type Item = &'a mut TwmRuntimeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node_id = self.stack.pop()?;

        // Collect and sort children via shared borrows before taking the mutable ref.
        let children = {
            let node = self.tree.nodes.get(&node_id)?;
            let mut children = node.children.clone();
            children.sort_by_key(|id| self.tree.nodes.get(id).map_or(0, |n| n.priority));
            children
        };
        for child in children.into_iter().rev() {
            self.stack.push(child);
        }

        // SAFETY: each NodeId appears at most once in the stack (tree has no cycles),
        // so we never hand out two &mut refs to the same node. We extend the lifetime
        // to 'a after all shared borrows of `self.tree` above are dropped.
        let node = unsafe {
            let ptr = self.tree.nodes.get_mut(&node_id)? as *mut TwmRuntimeNode;
            &mut *ptr
        };

        Some(node)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct TwmRuntimeNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub kind: TwmNodeKind,
    pub lifetime: TwmNodeLifetime,
    pub priority: u32,
    pub initial_grow_factor: f32,
    pub condition: Option<TwmCondition>,
    pub max_stack_size: Option<usize>,
    pub stack_policy: TwmStackPolicy,

    // Runtime-only
    pub grow_factor: f32,
    pub windows: Vec<WindowIdentity>,
    pub active_window: Option<WindowIdentity>,
    /// Inner rect (DWM visible bounds, no shadow). Must match `Window::inner_rect()` at all callsites.
    pub rect: Option<Rect>,
}

impl TwmRuntimeNode {
    pub fn from_plugin(node: &TwmPluginNode) -> Self {
        Self {
            id: 0,                // to be filled
            parent: None,         // to be filled
            children: Vec::new(), // to be filled
            kind: node.kind,
            lifetime: node.lifetime,
            priority: node.priority,
            initial_grow_factor: node.grow_factor,
            grow_factor: node.grow_factor,
            condition: node.condition.clone(),
            max_stack_size: node.max_stack_size,
            stack_policy: node.stack_policy,
            windows: Vec::new(),
            active_window: None,
            rect: None,
        }
    }

    fn accepts_windows(&self, ctx: &TwmConditionContext) -> bool {
        // 1. condition check (DSL rule)
        if let Some(cond) = &self.condition
            && !cond.evaluate(ctx)
        {
            return false;
        }

        // 2. structural rules
        match self.kind {
            TwmNodeKind::Leaf => self.windows.is_empty(),
            TwmNodeKind::Stack => {
                self.stack_policy == TwmStackPolicy::Auto
                    && match self.max_stack_size {
                        Some(max) => self.windows.len() < max,
                        None => true, // unlimited stack
                    }
            }
            // these never accept directly
            TwmNodeKind::Vertical | TwmNodeKind::Horizontal => false,
        }
    }

    fn accepts_windows_on_overflow(&self, ctx: &TwmConditionContext) -> bool {
        match self.kind {
            TwmNodeKind::Stack => {
                if let Some(cond) = &self.condition
                    && !cond.evaluate(ctx)
                {
                    return false;
                }

                self.stack_policy == TwmStackPolicy::AutoWhenOverflow
                    && match self.max_stack_size {
                        Some(max) => self.windows.len() < max,
                        None => true, // unlimited stack
                    }
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win(hwnd: WindowHandle) -> WindowIdentity {
        WindowIdentity {
            hwnd,
            pid: 1,
            process_created_at: 1,
        }
    }

    fn node(id: NodeId, kind: TwmNodeKind, children: Vec<NodeId>) -> TwmRuntimeNode {
        TwmRuntimeNode {
            id,
            parent: None,
            children,
            kind,
            lifetime: TwmNodeLifetime::Permanent,
            priority: 0,
            initial_grow_factor: 1.0,
            condition: None,
            max_stack_size: None,
            stack_policy: TwmStackPolicy::Manual,
            grow_factor: 1.0,
            windows: Vec::new(),
            active_window: None,
            rect: None,
        }
    }

    fn leaf(id: NodeId, windows: &[WindowHandle], lifetime: TwmNodeLifetime) -> TwmRuntimeNode {
        let mut node = node(id, TwmNodeKind::Leaf, vec![]);
        node.lifetime = lifetime;
        node.windows = windows.iter().map(|&h| win(h)).collect();
        node.active_window = node.windows.first().copied();
        node
    }

    /// Simulates a tree read from disk: runtime only fields are lost on serialization.
    fn roundtrip(tree: &TwmRuntimeTree) -> TwmRuntimeTree {
        serde_json::from_str(&serde_json::to_string(tree).unwrap()).unwrap()
    }

    fn tree(root: NodeId, nodes: Vec<TwmRuntimeNode>) -> TwmRuntimeTree {
        let mut tree = TwmRuntimeTree::new();
        tree.root = root;
        tree.nodes = nodes.into_iter().map(|n| (n.id, n)).collect();
        tree
    }

    #[test]
    fn restores_runtime_indexes_in_tree_order() {
        let stored = tree(
            1,
            vec![
                node(1, TwmNodeKind::Horizontal, vec![2, 3]),
                leaf(2, &[10], TwmNodeLifetime::Permanent),
                leaf(3, &[20], TwmNodeLifetime::Permanent),
            ],
        );
        let mut restored = roundtrip(&stored);
        assert!(restored.window_map.is_empty());

        assert!(restored.sanitize_structure());
        restored.restore_runtime_indexes();

        assert_eq!(restored.next_id, 4);
        assert_eq!(restored.node_of_window(10), Some(2));
        assert_eq!(restored.node_of_window(20), Some(3));

        // a reindex must keep the stored order
        let residual = restored.reindex_windows();
        assert!(residual.is_empty());
        assert_eq!(restored.node_of_window(10), Some(2));
        assert_eq!(restored.node_of_window(20), Some(3));
    }

    #[test]
    fn sanitize_structure_repairs_tree() {
        let mut root = node(1, TwmNodeKind::Horizontal, vec![2, 3, 99, 2]);
        root.windows = vec![win(1)]; // containers can't hold windows
        let mut duplicated = leaf(3, &[10, 30], TwmNodeLifetime::Permanent);
        duplicated.active_window = Some(win(77)); // not in the node
        let mut wrong_parent = leaf(2, &[10], TwmNodeLifetime::Permanent);
        wrong_parent.parent = Some(42);

        let mut t = tree(
            1,
            vec![
                root,
                wrong_parent,
                duplicated,
                leaf(50, &[500], TwmNodeLifetime::Permanent), // orphan
            ],
        );

        assert!(t.sanitize_structure());
        assert_eq!(t.nodes[&1].children, vec![2, 3]);
        assert!(t.nodes[&1].windows.is_empty());
        assert_eq!(t.nodes[&2].parent, Some(1));
        assert!(!t.nodes.contains_key(&50));
        assert_eq!(t.nodes[&2].windows, vec![win(10)]);
        assert_eq!(t.nodes[&3].windows, vec![win(30)]);
        assert_eq!(t.nodes[&3].active_window, Some(win(30)));
    }

    #[test]
    fn sanitize_structure_rejects_missing_root() {
        let mut t = tree(7, vec![leaf(1, &[10], TwmNodeLifetime::Permanent)]);
        assert!(!t.sanitize_structure());
        assert!(!TwmRuntimeTree::new().sanitize_structure());
    }

    #[test]
    fn retain_windows_drops_dead_windows_and_collapses() {
        let mut t = tree(
            1,
            vec![
                node(1, TwmNodeKind::Horizontal, vec![2, 3]),
                leaf(2, &[10], TwmNodeLifetime::Permanent),
                leaf(3, &[20], TwmNodeLifetime::Temporal),
            ],
        );
        t.sanitize_structure();
        t.restore_runtime_indexes();

        let residual = t.retain_windows(|w| w.hwnd != 20);
        assert!(residual.is_empty());
        assert!(!t.contains(20));
        assert!(t.contains(10));
        // the temporal leaf was drained and the single child container collapsed
        assert!(!t.nodes.contains_key(&3));
        assert_eq!(t.nodes[&t.root].kind, TwmNodeKind::Leaf);
        assert_eq!(t.node_of_window(10), Some(t.root));
    }
}
