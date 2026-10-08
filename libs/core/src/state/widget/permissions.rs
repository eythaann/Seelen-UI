use serde::{Deserialize, Serialize};

/// Commands that require explicit permission for third-party widgets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum WidgetPerm {
    Run,
    OpenFile,
    /// Write settings, toolbar/dock items or remove resources. Toolbar and dock items contain
    /// code executed by bundled widgets, so this must never be granted silently.
    ModifyConfiguration,
    KillProcesses,
    /// Focus, close, minimize, move or resize windows of other applications.
    ManageAppWindows,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum WidgetPermState {
    Allowed,
    Denied,
}
