#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
pub struct TwmPlugin {
    /// null means no tiling, only float layout
    pub structure: Option<TwmPluginNode>,
}

impl TwmPlugin {
    pub fn monocle() -> Self {
        Self {
            structure: Some(TwmPluginNode {
                kind: TwmNodeKind::Stack,
                max_stack_size: None, // unlimited
                ..Default::default()
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum TwmNodeLifetime {
    Temporal,
    #[default]
    Permanent,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum TwmStackPolicy {
    Manual,
    AutoWhenOverflow,
    #[default]
    Auto,
}

// ============== WmReservation ==============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum TwmReservation {
    Left,
    Right,
    Top,
    Bottom,
    Stack,
    Float,
}

// ============== TwmCondition ==============

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum TwmCondition {
    Compare {
        left: Operand,
        op: Comparator,
        right: serde_json::Value,
    },
    And(Box<TwmCondition>, Box<TwmCondition>),
    Or(Box<TwmCondition>, Box<TwmCondition>),
    Not(Box<TwmCondition>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
#[serde(rename_all = "kebab-case")]
pub enum Operand {
    TilingWindows,
    IsReindexing,
}

impl Operand {
    fn resolve(&self, ctx: &TwmConditionContext) -> serde_json::Value {
        match self {
            Operand::TilingWindows => ctx.tiling_windows.into(),
            Operand::IsReindexing => ctx.is_reindexing.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
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
    pub fn compare(&self, left: &serde_json::Value, right: &serde_json::Value) -> bool {
        use serde_json::Value;

        if let (Value::Number(l), Value::Number(r)) = (left, right) {
            let (Some(l), Some(r)) = (l.as_f64(), r.as_f64()) else {
                return false;
            };

            return match self {
                Comparator::Eq => l == r,
                Comparator::Ne => l != r,
                Comparator::Lt => l < r,
                Comparator::Le => l <= r,
                Comparator::Gt => l > r,
                Comparator::Ge => l >= r,
            };
        }

        match self {
            Comparator::Eq => left == right,
            Comparator::Ne => left != right,
            _ => false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TwmConditionContext {
    pub tiling_windows: usize,
    pub is_reindexing: bool,
}

impl TwmCondition {
    pub fn evaluate(&self, ctx: &TwmConditionContext) -> bool {
        match self {
            TwmCondition::Compare { left, op, right } => {
                let left = left.resolve(ctx);
                op.compare(&left, right)
            }
            TwmCondition::And(a, b) => a.evaluate(ctx) && b.evaluate(ctx),
            TwmCondition::Or(a, b) => a.evaluate(ctx) || b.evaluate(ctx),
            TwmCondition::Not(a) => !a.evaluate(ctx),
        }
    }
}
