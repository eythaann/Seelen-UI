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
                subtype: NodeSubtype,
                /// Order in how the tree will be traversed (1 = first, 2 = second, etc.)
                #[serde(default = "WmNode::default_priority")]
                priority: u32,
                /// How much of the remaining space this node will take
                #[serde(default = "WmNode::default_grow_factor")]
                grow_factor: f64,
                /// Math Condition for the node to be shown, e.g: n >= 3
                condition: Option<String>,
                $($rest)*
            }
        )*
    };
}

common_item! {
    struct WmVerticalNode {
        #[serde(default)]
        children: Vec<WmNode>,
    }
    struct WmHorizontalNode {
        #[serde(default)]
        children: Vec<WmNode>,
    }
    struct WmLeafNode {
        /// window handle (HWND) in the node
        handle: Option<isize>,
    }
    struct WmStackNode {
        /// active window handle (HWND) in the node
        active: Option<isize>,
        /// window handles (HWND) in the node
        #[serde(default)]
        handles: Vec<isize>,
    }
    struct WmFallbackNode {
        /// active window handle (HWND) in the node
        active: Option<isize>,
        /// window handles (HWND) in the node
        #[serde(default)]
        handles: Vec<isize>,
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

impl WmNode {
    fn default_subtype() -> NodeSubtype {
        NodeSubtype::Permanent
    }

    fn default_priority() -> u32 {
        1
    }

    fn default_grow_factor() -> f64 {
        1f64
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
