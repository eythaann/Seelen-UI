#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct TwmPlugin {
    pub structure: TwmPluginNode,
}

impl Default for TwmPlugin {
    fn default() -> Self {
        Self {
            structure: TwmPluginNode {
                kind: TwmNodeKind::Stack,
                max_stack_size: None,
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct TwmPluginNode {
    /// Type determines the behavior of the node
    #[serde(alias = "type")]
    pub kind: TwmNodeKind,
    /// Lifetime of the node
    pub lifetime: TwmNodeLifetime,
    /// Order in how the tree will be traversed (1 = first, 2 = second, etc.)
    pub priority: u32,
    /// How much of the remaining space this node will take
    pub grow_factor: f32,
    /// Condition for the node to be enabled
    pub condition: Option<TwmCondition>,
    /// Child nodes, this field is ignored for leaf and stack nodes.
    pub children: Vec<TwmPluginNode>,
    /// Max amount of windows in the stack. Set it to `null` for unlimited stack.\
    /// This field is ignored for non-stack nodes
    pub max_stack_size: Option<usize>,
    /// When to add new windows to the stack
    pub stack_policy: TwmStackPolicy,
}

impl Default for TwmPluginNode {
    fn default() -> Self {
        Self {
            kind: TwmNodeKind::Leaf,
            lifetime: Default::default(),
            priority: 1,
            grow_factor: 1.0,
            condition: None,
            children: Vec::new(),
            max_stack_size: Some(3),
            stack_policy: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum TwmNodeKind {
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
pub enum TwmNodeLifetime {
    Temporal,
    #[default]
    Permanent,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum TwmStackPolicy {
    Manual,
    AutoWhenOverflow,
    #[default]
    Auto,
}

// ============== WmReservation ==============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum TwmReservation {
    Left,
    Right,
    Top,
    Bottom,
    Stack,
    Float,
}

// ============== TwmCondition ==============

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum TwmCondition {
    Compare {
        left: Operand,
        op: Comparator,
        right: f64,
    },
    And(Box<TwmCondition>, Box<TwmCondition>),
    Or(Box<TwmCondition>, Box<TwmCondition>),
    Not(Box<TwmCondition>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
#[serde(rename_all = "kebab-case")]
pub enum Operand {
    TilingWindows,
    FloatingWindows,
    TotalWindows,
}

impl Operand {
    fn resolve(&self, ctx: &TwmConditionContext) -> f64 {
        match self {
            Operand::TilingWindows => ctx.tiling_windows as f64,
            Operand::FloatingWindows => ctx.floating_windows as f64,
            Operand::TotalWindows => ctx.total_windows as f64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
#[serde(rename_all = "kebab-case")]
pub enum Comparator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Comparator {
    pub fn compare(&self, left: f64, right: f64) -> bool {
        match self {
            Comparator::Eq => left == right,
            Comparator::Ne => left != right,
            Comparator::Lt => left < right,
            Comparator::Le => left <= right,
            Comparator::Gt => left > right,
            Comparator::Ge => left >= right,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TwmConditionContext {
    pub tiling_windows: usize,
    pub floating_windows: usize,
    pub total_windows: usize,
}

impl TwmCondition {
    pub fn evaluate(&self, ctx: &TwmConditionContext) -> bool {
        match self {
            TwmCondition::Compare { left, op, right } => {
                let left = left.resolve(ctx);
                op.compare(left, *right)
            }
            TwmCondition::And(a, b) => a.evaluate(ctx) && b.evaluate(ctx),
            TwmCondition::Or(a, b) => a.evaluate(ctx) || b.evaluate(ctx),
            TwmCondition::Not(a) => !a.evaluate(ctx),
        }
    }
}
