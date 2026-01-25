use std::{cell::Cell, collections::HashMap};

use crate::system_state::MonitorId;

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct WmRenderTree(pub HashMap<MonitorId, WindowManagerLayout>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WmNode {
    /// Type determines the behavior of the node
    #[serde(rename = "type")]
    pub kind: WmNodeKind,
    /// Lifetime of the node
    pub lifetime: WmNodeLifetime,
    /// Order in how the tree will be traversed (1 = first, 2 = second, etc.)
    pub priority: u32,
    /// How much of the remaining space this node will take
    pub grow_factor: Cell<f32>,
    /// Math Condition for the node to be shown, e.g: n >= 3
    pub condition: Option<String>,
    /// Active window handle (HWND) in the node.
    #[serde(skip_deserializing)]
    pub active: Option<isize>,
    /// Window handles (HWND) in the node.
    #[serde(skip_deserializing)]
    pub windows: Vec<isize>,
    /// Child nodes, this field is ignored for leaf and stack nodes.
    pub children: Vec<WmNode>,
    /// Max amount of windows in the stack. Set it to `null` for unlimited stack.\
    /// This field is ignored for non-stack nodes
    pub max_stack_size: Option<usize>,
}

unsafe impl Send for WmNode {}
unsafe impl Sync for WmNode {}

impl WmNode {
    pub fn len(&self) -> usize {
        match self.kind {
            WmNodeKind::Leaf | WmNodeKind::Stack => self.windows.len(),
            WmNodeKind::Vertical | WmNodeKind::Horizontal => {
                self.children.iter().map(|n| n.len()).sum()
            }
        }
    }

    pub fn capacity(&self) -> usize {
        match self.kind {
            WmNodeKind::Leaf => 1usize,
            WmNodeKind::Stack => self.max_stack_size.unwrap_or(usize::MAX),
            WmNodeKind::Vertical | WmNodeKind::Horizontal => {
                let mut total = 0usize;
                for n in &self.children {
                    total = total.saturating_add(n.capacity());
                }
                total
            }
        }
    }

    pub fn is_full(&self) -> bool {
        match self.kind {
            WmNodeKind::Leaf => !self.windows.is_empty(),
            WmNodeKind::Stack => self.max_stack_size.is_some_and(|max| self.len() >= max),
            WmNodeKind::Vertical | WmNodeKind::Horizontal => {
                self.children.iter().all(|n| n.is_full())
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for WmNode {
    fn default() -> Self {
        Self {
            kind: WmNodeKind::Leaf,
            lifetime: WmNodeLifetime::Permanent,
            priority: 1,
            grow_factor: Cell::new(1.0),
            condition: None,
            active: None,
            windows: Vec::new(),
            children: Vec::new(),
            max_stack_size: Some(3),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WmNodeKind {
    /// node will not grow, this is the final node.
    Leaf,
    /// node will grow on z-axis
    Stack,
    /// node will grow on y-axis
    Vertical,
    /// node will grow on x-axis
    Horizontal,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum WmNodeLifetime {
    Temporal,
    #[default]
    Permanent,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct WindowManagerLayout {
    pub structure: WmNode,
    #[serde(skip_deserializing)]
    pub floating_windows: Vec<isize>,
}

impl Default for WindowManagerLayout {
    fn default() -> Self {
        Self {
            structure: WmNode {
                kind: WmNodeKind::Stack,
                max_stack_size: None,
                ..Default::default()
            },
            floating_windows: Vec::new(),
        }
    }
}
