use std::cell::Cell;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

macro_rules! common_item {
    (
        $(
            struct $name:ident {
                $($rest:tt)*
            }
        )*
    ) => {
        $(
            #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
            #[serde(rename_all = "camelCase")]
            pub struct $name {
                #[serde(default = "WmNode::default_subtype")]
                pub subtype: NodeSubtype,
                /// Order in how the tree will be traversed (1 = first, 2 = second, etc.)
                #[serde(default = "WmNode::default_priority")]
                pub priority: u32,
                /// How much of the remaining space this node will take
                #[serde(default = "WmNode::default_grow_factor")]
                pub grow_factor: Cell<f32>,
                /// Math Condition for the node to be shown, e.g: n >= 3
                pub condition: Option<String>,
                $($rest)*
            }
        )*
    };
}

common_item! {
    struct WmVerticalNode {
        pub children: Vec<WmNode>,
    }
    struct WmHorizontalNode {
        pub children: Vec<WmNode>,
    }
    struct WmLeafNode {
        /// window handle (HWND) in the node
        pub handle: Option<isize>,
    }
    struct WmStackNode {
        /// active window handle (HWND) in the node
        #[serde(skip_deserializing)]
        pub active: Option<isize>,
        /// window handles (HWND) in the node
        #[serde(skip_deserializing)]
        pub handles: Vec<isize>,
    }
    struct WmFallbackNode {
        /// active window handle (HWND) in the node
        #[serde(skip_deserializing)]
        pub active: Option<isize>,
        /// window handles (HWND) in the node
        #[serde(skip_deserializing)]
        pub handles: Vec<isize>,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum WmNode {
    Vertical(WmVerticalNode),
    Horizontal(WmHorizontalNode),
    Leaf(WmLeafNode),
    Stack(WmStackNode),
    Fallback(WmFallbackNode),
}

fn format_children(children: &[WmNode]) -> String {
    let mut result = Vec::new();
    for child in children {
        result.push(child.to_string());
    }
    result.join(", ")
}

impl std::fmt::Display for WmNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WmNode::Vertical(node) => write!(f, "Vertical [{}]", format_children(&node.children)),
            WmNode::Horizontal(node) => {
                write!(f, "Horizontal [{}]", format_children(&node.children))
            }
            WmNode::Leaf(node) => write!(f, "Leaf({:?})", node.handle),
            WmNode::Stack(node) => write!(f, "Stack({:?})", node.handles),
            WmNode::Fallback(node) => write!(f, "Fallback({:?})", node.handles),
        }
    }
}

impl WmNode {
    fn default_subtype() -> NodeSubtype {
        NodeSubtype::Permanent
    }

    fn default_priority() -> u32 {
        1
    }

    fn default_grow_factor() -> Cell<f32> {
        Cell::new(1.0)
    }

    pub fn priority(&self) -> u32 {
        match self {
            WmNode::Leaf(n) => n.priority,
            WmNode::Stack(n) => n.priority,
            WmNode::Fallback(n) => n.priority,
            WmNode::Vertical(n) => n.priority,
            WmNode::Horizontal(n) => n.priority,
        }
    }

    pub fn grow_factor(&self) -> &Cell<f32> {
        match self {
            WmNode::Leaf(n) => &n.grow_factor,
            WmNode::Stack(n) => &n.grow_factor,
            WmNode::Fallback(n) => &n.grow_factor,
            WmNode::Vertical(n) => &n.grow_factor,
            WmNode::Horizontal(n) => &n.grow_factor,
        }
    }

    pub fn condition(&self) -> Option<&String> {
        match self {
            WmNode::Leaf(n) => n.condition.as_ref(),
            WmNode::Stack(n) => n.condition.as_ref(),
            WmNode::Fallback(n) => n.condition.as_ref(),
            WmNode::Vertical(n) => n.condition.as_ref(),
            WmNode::Horizontal(n) => n.condition.as_ref(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            WmNode::Leaf(n) => n.handle.is_some() as usize,
            WmNode::Stack(n) => n.handles.len(),
            WmNode::Fallback(n) => n.handles.len(),
            WmNode::Vertical(n) => n.children.iter().map(Self::len).sum(),
            WmNode::Horizontal(n) => n.children.iter().map(Self::len).sum(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct WManagerLayoutInfo {
    /// Display name of the layout
    pub display_name: String,
    /// Author of the layout
    pub author: String,
    /// Description of the layout
    pub description: String,
    /// Filename of the layout, is overridden by the program on load.
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum NodeSubtype {
    Temporal,
    Permanent,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum NoFallbackBehavior {
    Float,
    Unmanaged,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowManagerLayout {
    pub info: WManagerLayoutInfo,
    pub structure: WmNode,
    pub no_fallback_behavior: NoFallbackBehavior,
}

impl Default for WindowManagerLayout {
    fn default() -> Self {
        Self {
            info: Default::default(),
            structure: WmNode::Fallback(WmFallbackNode {
                subtype: WmNode::default_subtype(),
                priority: WmNode::default_priority(),
                grow_factor: WmNode::default_grow_factor(),
                condition: None,
                active: None,
                handles: vec![],
            }),
            no_fallback_behavior: NoFallbackBehavior::Float,
        }
    }
}
