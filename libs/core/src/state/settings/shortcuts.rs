use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::state::Widget;

/// Declaration for a system-level shortcut (not attached to any specific widget definition).
/// Hardcoded in Rust; exposed to the frontend via the `StateGetSystemShortcuts` command.
/// The user can override keys via `SluShortcutsSettings.shortcuts`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct SystemShortcutDeclaration {
    pub id: String,
    pub command: Vec<String>,
    pub label: String,
    pub default_keys: Vec<String>,
    /// If true, user cannot change the keys for this shortcut.
    pub readonly: bool,
}

/// Minimal struct sent to the service after the background has resolved all shortcut overrides.
/// Contains only what the service needs to register hotkeys — no widget info, no enums.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct ResolvedShortcut {
    pub command: Vec<String>,
    pub keys: Vec<String>,
}

/// User-facing shortcut settings.
/// - `enabled`: global on/off toggle.
/// - `shortcuts`: key overrides for **system-level** shortcut declarations (`id -> keys`).
///   Widget shortcut overrides live inside each widget's `$shortcuts` in `SettingsByWidget`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct SluShortcutsSettings {
    pub enabled: bool,
    pub shortcuts: HashMap<String, Vec<String>>,
}

impl Default for SluShortcutsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            shortcuts: HashMap::new(),
        }
    }
}

// =====================================================================
// Helpers
// =====================================================================

macro_rules! cmd {
    ($($arg:expr),+ $(,)?) => {
        vec![$($arg.to_string()),+]
    };
}

macro_rules! decl {
    ($id:expr, $label:expr, $command:expr, $default_keys:expr, readonly) => {
        SystemShortcutDeclaration {
            id: $id.to_string(),
            label: $label.to_string(),
            command: $command,
            default_keys: $default_keys,
            readonly: true,
        }
    };
    ($id:expr, $label:expr, $command:expr, $default_keys:expr) => {
        SystemShortcutDeclaration {
            id: $id.to_string(),
            label: $label.to_string(),
            command: $command,
            default_keys: $default_keys,
            readonly: false,
        }
    };
}

// =====================================================================
// System shortcut declarations (hardcoded defaults)
// =====================================================================

/// Returns all hardcoded system-level shortcut declarations.
/// These are shortcuts that are either:
/// - Not tied to any single widget (VD system, misc), or
/// - System overrides that must not be freely editable (dock Win+N).
pub fn system_shortcut_declarations() -> Vec<SystemShortcutDeclaration> {
    // ---- Virtual Desktop system overrides (readonly) ----
    let mut decls = vec![
        decl!(
            "vd-create-workspace",
            "t:shortcuts.labels.create_new_workspace",
            cmd!["vd", "create-new-workspace"],
            cmd!["Ctrl", "Win", "D"],
            readonly
        ),
        decl!(
            "vd-destroy-workspace",
            "t:shortcuts.labels.destroy_current_workspace",
            cmd!["vd", "destroy-current-workspace"],
            cmd!["Ctrl", "Win", "F4"],
            readonly
        ),
        decl!(
            "vd-switch-next",
            "t:shortcuts.labels.switch_to_next_workspace",
            cmd!["vd", "switch-next"],
            cmd!["Ctrl", "Win", "Right"],
            readonly
        ),
        decl!(
            "vd-switch-prev",
            "t:shortcuts.labels.switch_to_previous_workspace",
            cmd!["vd", "switch-prev"],
            cmd!["Ctrl", "Win", "Left"],
            readonly
        ),
    ];

    // ---- Workspace switch/move/send (user-configurable) ----
    for index in 0..10usize {
        let digit = if index == 9 {
            "0".to_string()
        } else {
            format!("{}", index + 1)
        };
        decls.push(SystemShortcutDeclaration {
            id: format!("vd-switch-to-{}", index),
            label: format!("t:shortcuts.labels.switch_workspace:{}", index + 1),
            command: cmd!["vd", "switch-workspace", index],
            default_keys: vec!["Alt".to_string(), digit.clone()],
            readonly: false,
        });
        decls.push(SystemShortcutDeclaration {
            id: format!("vd-move-to-{}", index),
            label: format!("t:shortcuts.labels.move_to_workspace:{}", index + 1),
            command: cmd!["vd", "move-to-workspace", index],
            default_keys: vec!["Alt".to_string(), "Shift".to_string(), digit.clone()],
            readonly: false,
        });
        decls.push(SystemShortcutDeclaration {
            id: format!("vd-send-to-{}", index),
            label: format!("t:shortcuts.labels.send_to_workspace:{}", index + 1),
            command: cmd!["vd", "send-to-workspace", index],
            default_keys: vec!["Win".to_string(), "Shift".to_string(), digit],
            readonly: false,
        });
    }

    // ---- Misc (readonly) ----
    decls.push(decl!(
        "service-force-restart",
        "t:shortcuts.labels.misc_force_restart",
        cmd!["service", "force-restart"],
        cmd!["Ctrl", "Win", "Alt", "R"],
        readonly
    ));
    decls.push(decl!(
        "service-force-quit",
        "t:shortcuts.labels.misc_force_quit",
        cmd!["service", "force-quit"],
        cmd!["Ctrl", "Win", "Alt", "K"],
        readonly
    ));

    decls
}

// =====================================================================
// Resolution
// =====================================================================

use super::Settings;

/// Resolves all shortcut declarations (widget-declared + system-hardcoded) against
/// user-configured overrides, respecting widget enabled state.
///
/// Returns a flat list of `ResolvedShortcut` ready to be sent to the service.
/// Returns an empty `Vec` if `settings.shortcuts.enabled` is false.
pub fn resolve_shortcuts(settings: &Settings, widgets: &[&Widget]) -> Vec<ResolvedShortcut> {
    if !settings.shortcuts.enabled {
        return vec![];
    }

    let mut resolved = Vec::new();

    // 1. Widget-declared shortcuts
    for widget in widgets {
        if !settings.is_widget_enabled(&widget.id) {
            continue;
        }

        let overrides = settings.by_widget.get_shortcut_overrides(&widget.id);
        for decl in &widget.shortcuts {
            let keys = if decl.readonly {
                decl.default_keys.clone()
            } else {
                overrides
                    .get(&decl.id)
                    .cloned()
                    .unwrap_or_else(|| decl.default_keys.clone())
            };
            if keys.is_empty() {
                continue;
            }
            resolved.push(ResolvedShortcut {
                command: decl.command.clone(),
                keys,
            });
        }
    }

    // 2. System-level declarations
    let system_overrides = &settings.shortcuts.shortcuts;
    for decl in system_shortcut_declarations() {
        let keys = if decl.readonly {
            decl.default_keys.clone()
        } else {
            system_overrides
                .get(&decl.id)
                .cloned()
                .unwrap_or_else(|| decl.default_keys.clone())
        };
        if keys.is_empty() {
            continue;
        }
        resolved.push(ResolvedShortcut {
            command: decl.command.clone(),
            keys,
        });
    }

    resolved
}
