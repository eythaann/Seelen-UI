use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

use parking_lot::Mutex;
use seelen_core::resource::WidgetId;
use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    resources::RESOURCES,
    utils::constants::SEELEN_COMMON,
    widgets::webview::WidgetWebviewLabel,
};

// =============================================================================
// Types
// =============================================================================

/// Commands that require explicit permission for third-party widgets.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetPerm {
    Run,
    OpenFile,
}

impl WidgetPerm {
    fn i18n_label(&self) -> String {
        match self {
            WidgetPerm::Run => t!("widget_permissions.perm_run"),
            WidgetPerm::OpenFile => t!("widget_permissions.perm_open_file"),
        }
        .to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetPermState {
    Allowed,
    Denied,
}

#[derive(Default, Serialize, Deserialize)]
pub struct WidgetPermissions(HashMap<WidgetId, HashMap<WidgetPerm, WidgetPermState>>);

// =============================================================================
// Manager
// =============================================================================

pub struct PermissionsManager {
    data: Mutex<WidgetPermissions>,
    /// Serializes dialog display so only one permission dialog appears at a time.
    dialog_lock: Mutex<()>,
    path: PathBuf,
}

pub static WIDGET_PERMISSIONS: LazyLock<PermissionsManager> = LazyLock::new(|| {
    let path = SEELEN_COMMON.app_data_dir().join("permissions.json");
    PermissionsManager::load(path)
});

impl PermissionsManager {
    fn load(path: PathBuf) -> Self {
        let data = std::fs::File::open(&path)
            .ok()
            .and_then(|f| serde_json::from_reader(f).ok())
            .unwrap_or_default();

        Self {
            data: Mutex::new(data),
            dialog_lock: Mutex::new(()),
            path,
        }
    }

    fn save(&self) -> Result<()> {
        let data = self.data.lock();
        let file = std::fs::File::create(&self.path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &*data)?;
        Ok(())
    }

    /// Returns `Some(true)` if previously allowed, `Some(false)` if denied, `None` if unknown.
    fn is_resolved(&self, widget_id: &WidgetId, command: &WidgetPerm) -> Option<bool> {
        let data = self.data.lock();
        match data.0.get(widget_id).and_then(|perms| perms.get(command)) {
            Some(WidgetPermState::Allowed) => Some(true),
            Some(WidgetPermState::Denied) => Some(false),
            None => None,
        }
    }

    fn persist_decision(&self, widget_id: WidgetId, command: WidgetPerm, granted: bool) {
        let state = if granted {
            WidgetPermState::Allowed
        } else {
            WidgetPermState::Denied
        };
        let mut data = self.data.lock();
        data.0.entry(widget_id).or_default().insert(command, state);
    }

    /// Main entry point. Grants permission immediately for bundled widgets.
    /// For third-party widgets checks the stored decision or prompts the user.
    pub fn request(&self, widget_id: &WidgetId, command: WidgetPerm) -> Result<()> {
        // Bundled widgets always have permission.
        if RESOURCES
            .widgets
            .read(widget_id, |_, w| w.metadata.internal.bundled)
            .unwrap_or(false)
        {
            return Ok(());
        }

        // Fast path: decision already recorded.
        if let Some(granted) = self.is_resolved(widget_id, &command) {
            return Self::decision_to_result(granted, widget_id, &command);
        }

        // Slow path: show dialog (serialized so only one dialog appears at a time).
        let _dialog_guard = self.dialog_lock.lock();

        // Re-check after acquiring the lock – another thread may have resolved it.
        if let Some(granted) = self.is_resolved(widget_id, &command) {
            return Self::decision_to_result(granted, widget_id, &command);
        }

        let lang = rust_i18n::locale();
        let widget_name = RESOURCES
            .widgets
            .read(widget_id, |_, w| {
                w.metadata.display_name.get(&lang).to_string()
            })
            .unwrap_or_else(|| widget_id.to_string());

        let message = t!(
            "widget_permissions.request_description",
            widget_name = widget_name,
            command = command.i18n_label()
        );

        let granted = get_app_handle()
            .dialog()
            .message(message)
            .title(t!("widget_permissions.request_title"))
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::YesNo)
            .blocking_show();

        self.persist_decision(widget_id.clone(), command.clone(), granted);
        self.save().log_error();

        Self::decision_to_result(granted, widget_id, &command)
    }

    fn decision_to_result(granted: bool, widget_id: &WidgetId, command: &WidgetPerm) -> Result<()> {
        if granted {
            Ok(())
        } else {
            Err(format!(
                "Widget '{}' does not have permission to '{}'.",
                widget_id,
                command.i18n_label()
            )
            .into())
        }
    }
}

// =============================================================================
// Public helper for Tauri command handlers
// =============================================================================

/// Resolves the calling widget from the webview label and checks (or requests)
/// permission for `command`. Returns `Ok(())` if access is granted.
pub fn request_widget_permission(
    webview: &tauri::WebviewWindow,
    command: WidgetPerm,
) -> Result<()> {
    let label = WidgetWebviewLabel::try_from_raw(webview.label())
        .map_err(|_| "Permission denied: caller is not a widget webview.")?;
    WIDGET_PERMISSIONS.request(&label.widget_id, command)
}

/// Dev-only command: simulates a permission request for any widget ID and perm.
/// Follows the same flow as a real request (checks cache, shows dialog, persists result).
#[tauri::command(async)]
pub fn simulate_perm(widget_id: String, perm: WidgetPerm) -> Result<()> {
    WIDGET_PERMISSIONS.request(&WidgetId::from(widget_id.as_str()), perm)
}
