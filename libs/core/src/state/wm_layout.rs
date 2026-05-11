use std::collections::HashMap;

use crate::{
    state::{
        twm::{
            TwmCondition, TwmConditionContext, TwmNodeKind, TwmNodeLifetime, TwmPlugin,
            TwmPluginNode, TwmReservation, TwmStackPolicy,
        },
        WorkspaceId,
    },
    Rect,
};

pub type NodeId = u64;
pub type WindowId = isize;

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct TwmGlobalRuntimeTree {
    pub workspaces: HashMap<WorkspaceId, TwmRuntimeTree>,
}

impl TwmGlobalRuntimeTree {
    pub fn contains(&self, window_id: &WindowId) -> bool {
        self.workspaces
            .iter()
            .any(|(_, tree)| tree.contains(window_id))
    }

    pub fn is_tiled(&self, window_id: &WindowId) -> bool {
        self.workspaces
            .iter()
            .any(|(_, tree)| tree.is_tiled(window_id))
    }

    pub fn is_floating(&self, window_id: &WindowId) -> bool {
        self.workspaces
            .iter()
            .any(|(_, tree)| tree.is_floating(window_id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct TwmRuntimeTree {
    pub next_id: NodeId,
    pub root: NodeId,
    pub nodes: HashMap<NodeId, TwmRuntimeNode>,
    pub window_map: HashMap<WindowId, WindowLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub enum WindowLocation {
    Tiled(NodeId),
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
        let root_id = runtime.insert_plugin_node(&tree.structure, None);
        runtime.root = root_id;
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

    pub fn contains(&self, window_id: &WindowId) -> bool {
        self.window_map.contains_key(window_id)
    }

    pub fn is_tiled(&self, id: &WindowId) -> bool {
        matches!(self.window_map.get(id), Some(WindowLocation::Tiled(_)))
    }

    pub fn is_floating(&self, id: &WindowId) -> bool {
        matches!(self.window_map.get(id), Some(WindowLocation::Floating))
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
        let mut floating_windows = 0;
        let mut total_windows = 0;

        for window in self.window_map.values() {
            match window {
                WindowLocation::Tiled(_) => tiling_windows += 1,
                WindowLocation::Floating => floating_windows += 1,
            }
            total_windows += 1;
        }

        TwmConditionContext {
            tiling_windows,
            floating_windows,
            total_windows,
        }
    }

    /// returns true if the window was added, false in case of overflow
    fn try_add_window(&mut self, window_id: WindowId, ctx: &TwmConditionContext) -> bool {
        if let Some(node_id) = self.iter().find(|n| n.accepts_windows(ctx)).map(|n| n.id) {
            let node = self.nodes.get_mut(&node_id).unwrap();
            node.windows.push(window_id);
            node.active_window = Some(window_id);
            self.window_map
                .insert(window_id, WindowLocation::Tiled(node_id));
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
            self.window_map
                .insert(window_id, WindowLocation::Tiled(node_id));
            return true;
        }

        false
    }

    pub fn drain_tiled(&mut self) -> Vec<WindowId> {
        let mut drained = Vec::new();
        for node in self.iter_mut() {
            drained.append(&mut node.windows);
            node.active_window = None;
        }
        self.window_map
            .retain(|_, location| matches!(location, WindowLocation::Floating));
        drained
    }

    /// reindexes windows to handle logical condition like `managed < 4` and returns residual windows
    pub fn reindex_windows(&mut self) -> Vec<WindowId> {
        let ctx = self.get_context();

        // Drain only Leaf nodes and single-window stacks.
        // Multi-window stacks (≥2 windows) are skipped — they stay where they are.
        let mut drained: Vec<isize> = Vec::new();
        for node in self.iter_mut() {
            if node.kind == TwmNodeKind::Stack && node.windows.len() >= 2 {
                continue;
            }
            drained.append(&mut node.windows);
            node.active_window = None;
        }

        for window in &drained {
            self.window_map.remove(window);
        }

        let mut overflow = Vec::new();
        for window in drained {
            if !self.try_add_window(window, &ctx) {
                overflow.push(window);
            }
        }

        // Collapse Manual stacks that are now no longer needed
        for node in self.nodes.values_mut() {
            if node.kind == TwmNodeKind::Stack
                && node.stack_policy == TwmStackPolicy::Manual
                && node.windows.len() <= 1
            {
                node.kind = TwmNodeKind::Leaf;
            }
        }

        overflow
    }

    /// returns residual windows that should be added to floating
    pub fn add_to_tiled(&mut self, window_id: WindowId) -> Vec<WindowId> {
        let ctx = self.get_context();
        if !self.try_add_window(window_id, &ctx) {
            return vec![window_id];
        }
        self.reindex_windows()
    }

    pub fn add_to_floating(&mut self, window_id: WindowId) {
        self.window_map.insert(window_id, WindowLocation::Floating);
    }

    pub fn remove_window(&mut self, window_id: &WindowId) -> Vec<isize> {
        let Some(location) = self.window_map.remove(window_id) else {
            return Vec::new();
        };

        match location {
            WindowLocation::Tiled(node_id) => {
                let node = self.nodes.get_mut(&node_id).unwrap();
                node.windows.retain(|w| w != window_id);
                if node.active_window == Some(*window_id) {
                    node.active_window = node.windows.first().copied();
                }
            }
            WindowLocation::Floating => {}
        }
        self.reindex_windows()
    }

    pub fn has_any_windows(&self, node_id: NodeId) -> bool {
        let node = &self.nodes[&node_id];
        if !node.windows.is_empty() {
            return true;
        }
        node.children.iter().any(|&c| self.has_any_windows(c))
    }

    pub fn node_of_window(&self, window_id: &WindowId) -> Option<NodeId> {
        match self.window_map.get(window_id)? {
            WindowLocation::Tiled(node_id) => Some(*node_id),
            WindowLocation::Floating => None,
        }
    }

    pub fn face_of_node(&self, node_id: NodeId) -> Option<WindowId> {
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

    pub fn node_is_stack(&self, window_id: &WindowId) -> bool {
        self.node_of_window(window_id)
            .and_then(|id| self.nodes.get(&id))
            .map(|n| n.kind == TwmNodeKind::Stack)
            .unwrap_or(false)
    }

    pub fn swap_nodes_by_windows(&mut self, a: WindowId, b: WindowId) {
        let node_a = match self.window_map.get(&a) {
            Some(WindowLocation::Tiled(id)) => *id,
            _ => return,
        };
        let node_b = match self.window_map.get(&b) {
            Some(WindowLocation::Tiled(id)) => *id,
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

        let windows_a: Vec<WindowId> = self.nodes[&node_a].windows.clone();
        let windows_b: Vec<WindowId> = self.nodes[&node_b].windows.clone();
        for w in windows_a {
            self.window_map.insert(w, WindowLocation::Tiled(node_a));
        }
        for w in windows_b {
            self.window_map.insert(w, WindowLocation::Tiled(node_b));
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

    pub fn siblings_at_side(
        &self,
        window_id: &WindowId,
        match_horizontal: bool,
        want_before: bool,
    ) -> Vec<NodeId> {
        let Some(mut current_id) = self.node_of_window(window_id) else {
            return vec![];
        };
        let wanted_kind = if match_horizontal {
            TwmNodeKind::Horizontal
        } else {
            TwmNodeKind::Vertical
        };

        loop {
            let Some(parent_id) = self.nodes[&current_id].parent else {
                return vec![];
            };
            let parent = &self.nodes[&parent_id];
            if parent.kind == wanted_kind {
                let child_idx = parent
                    .children
                    .iter()
                    .position(|&c| c == current_id)
                    .unwrap();
                let siblings: Vec<NodeId> = parent
                    .children
                    .iter()
                    .enumerate()
                    .filter(|(idx, &c)| {
                        let correct_side = if want_before {
                            *idx < child_idx
                        } else {
                            *idx > child_idx
                        };
                        *idx != child_idx && correct_side && self.has_any_windows(c)
                    })
                    .map(|(_, &c)| c)
                    .collect();
                if !siblings.is_empty() {
                    return siblings;
                }
            }
            current_id = parent_id;
        }
    }

    /// Splits `node_id` by inserting a new intermediate Horizontal (Left/Right)
    /// or Vertical (Top/Bottom) container, then places `new_window` in a freshly
    /// created sibling Leaf on the requested side.
    /// Returns `false` if `node_id` does not exist in this tree (caller falls back).
    pub fn split_node_for_reservation(
        &mut self,
        node_id: NodeId,
        side: TwmReservation,
        new_window: WindowId,
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
        self.window_map
            .insert(new_window, WindowLocation::Tiled(new_leaf_id));
        true
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
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
    pub windows: Vec<WindowId>,
    pub active_window: Option<WindowId>,
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
        if let Some(cond) = &self.condition {
            if !cond.evaluate(ctx) {
                return false;
            }
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
                if let Some(cond) = &self.condition {
                    if !cond.evaluate(ctx) {
                        return false;
                    }
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
