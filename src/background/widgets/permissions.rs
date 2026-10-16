use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{LazyLock, Once, mpsc},
};

use seelen_core::{handlers::SeelenEvent, resource::WidgetId, state::WidgetPermState};

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub use seelen_core::state::WidgetPerm;

use crate::{
    app::{emit_to_webviews, get_app_handle},
    error::{Result, ResultLogExt},
    event_manager,
    resources::RESOURCES,
    utils::{
        atomic_write_file, constants::SEELEN_COMMON, lock_free::TracedMutex, spawn_named_thread,
    },
    widgets::webview::WidgetWebviewLabel,
};

type WidgetPermissions = HashMap<WidgetId, HashMap<WidgetPerm, WidgetPermState>>;

// =============================================================================
// Manager
// =============================================================================

pub static WIDGET_PERMISSIONS: LazyLock<PermissionsManager> = LazyLock::new(|| {
    let path = SEELEN_COMMON.app_data_dir().join("permissions.json");
    let manager = PermissionsManager::load(path);
    // widgets could have been removed while the app was closed
    manager.remove_uninstalled_widgets().log_error();
    manager
});

/// Permission dialogs are shown one at a time by a dedicated thread, so no lock is held
/// while waiting for the user. Requests queued behind a dialog re-check the stored
/// decision first, so spamming a command can't open more than one dialog for it.
static DIALOG_QUEUE: LazyLock<mpsc::Sender<DialogRequest>> = LazyLock::new(|| {
    let (tx, rx) = mpsc::channel::<DialogRequest>();
    spawn_named_thread("Permission Dialogs", move || {
        for request in rx {
            // a panic must not kill the queue: that request is denied (and not
            // persisted) by dropping its reply, the next ones keep working
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                WIDGET_PERMISSIONS.resolve_with_dialog(&request.widget_id, &request.command)
            }));
            if let Ok(granted) = result {
                // the requester may have given up, nothing to do then
                let _ = request.reply.send(granted);
            }
        }
    });
    tx
});

pub struct PermissionsManager {
    data: TracedMutex<WidgetPermissions>,
    path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum PermissionsEvent {
    Changed,
}

event_manager!(PermissionsManager, PermissionsEvent);

impl PermissionsManager {
    fn load(path: PathBuf) -> Self {
        let data = std::fs::File::open(&path)
            .ok()
            .and_then(|f| serde_json::from_reader(f).ok())
            .unwrap_or_default();

        Self {
            data: TracedMutex::new(data),
            path,
        }
    }

    fn save(&self) -> Result<()> {
        let json = serde_json::to_vec_pretty(&*self.data.lock())?;
        atomic_write_file(&self.path, &json)?;
        Self::send(PermissionsEvent::Changed);
        Ok(())
    }

    pub fn get_all(&self) -> WidgetPermissions {
        self.data.lock().clone()
    }

    /// Replaces all stored decisions (used by the settings app to review them).
    pub fn set_all(&self, mut permissions: WidgetPermissions) -> Result<()> {
        retain_installed_widgets(&mut permissions);
        *self.data.lock() = permissions;
        self.save()
    }

    /// Drops the decisions of widgets that are no longer installed, so a widget
    /// installed later with the same id has to ask again.
    pub fn remove_uninstalled_widgets(&self) -> Result<()> {
        let removed = retain_installed_widgets(&mut self.data.lock());
        if removed {
            self.save()?;
        }
        Ok(())
    }

    /// Returns `Some(true)` if previously allowed, `Some(false)` if denied, `None` if unknown.
    fn is_resolved(&self, widget_id: &WidgetId, command: &WidgetPerm) -> Option<bool> {
        let data = self.data.lock();
        match data.get(widget_id).and_then(|perms| perms.get(command)) {
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
        data.entry(widget_id).or_default().insert(command, state);
    }

    /// Main entry point. Grants permission immediately for bundled widgets.
    /// For third-party widgets checks the stored decision or prompts the user.
    pub fn request(&self, widget_id: &WidgetId, command: WidgetPerm) -> Result<()> {
        // Bundled widgets always have permission.
        if RESOURCES
            .widgets
            .read_sync(widget_id, |_, w| w.metadata.internal.bundled)
            .unwrap_or(false)
        {
            return Ok(());
        }

        // Fast path: decision already recorded.
        if let Some(granted) = self.is_resolved(widget_id, &command) {
            return Self::decision_to_result(granted, widget_id, &command);
        }

        // Slow path: queue a dialog and wait for the user's answer.
        // If the dialog thread is gone (panicked), deny by default.
        let (reply, answer) = mpsc::channel();
        DIALOG_QUEUE
            .send(DialogRequest {
                widget_id: widget_id.clone(),
                command: command.clone(),
                reply,
            })
            .map_err(|_| "Permission dialog queue is not available.")?;
        let granted = answer
            .recv()
            .map_err(|_| "Permission dialog closed without an answer.")?;

        Self::decision_to_result(granted, widget_id, &command)
    }

    /// Runs on the dialog thread only, one request at a time.
    fn resolve_with_dialog(&self, widget_id: &WidgetId, command: &WidgetPerm) -> bool {
        // A request queued earlier may have resolved it already.
        if let Some(granted) = self.is_resolved(widget_id, command) {
            return granted;
        }

        let lang = rust_i18n::locale();
        let widget_name = RESOURCES
            .widgets
            .read_sync(widget_id, |_, w| {
                w.metadata.display_name.get(&lang).to_string()
            })
            .unwrap_or_else(|| widget_id.to_string());

        let message = t!(
            "widget_permissions.request_description",
            widget_name = widget_name,
            command = perm_i18n_label(command)
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
        granted
    }

    fn decision_to_result(granted: bool, widget_id: &WidgetId, command: &WidgetPerm) -> Result<()> {
        if granted {
            Ok(())
        } else {
            Err(format!(
                "Widget '{}' does not have permission to '{}'.",
                widget_id,
                perm_i18n_label(command)
            )
            .into())
        }
    }
}

struct DialogRequest {
    widget_id: WidgetId,
    command: WidgetPerm,
    reply: mpsc::Sender<bool>,
}

// =============================================================================
// Utilities
// =============================================================================

/// Returns true if any entry was removed. Uses the raw loaded widgets (not the
/// session-filtered list) so premium widgets keep their decisions when the session ends.
fn retain_installed_widgets(permissions: &mut WidgetPermissions) -> bool {
    let before = permissions.len();
    permissions.retain(|id, _| RESOURCES.widgets.read_sync(id, |_, _| ()).is_some());
    permissions.len() != before
}

fn perm_i18n_label(perm: &WidgetPerm) -> String {
    match perm {
        WidgetPerm::Run => t!("widget_permissions.perm_run"),
        WidgetPerm::OpenFile => t!("widget_permissions.perm_open_file"),
        WidgetPerm::ModifyConfiguration => t!("widget_permissions.perm_modify_configuration"),
        WidgetPerm::KillProcesses => t!("widget_permissions.perm_kill_processes"),
        WidgetPerm::ManageAppWindows => t!("widget_permissions.perm_manage_app_windows"),
    }
    .to_string()
}

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

fn get_manager() -> &'static PermissionsManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        PermissionsManager::subscribe(|_event: PermissionsEvent| {
            emit_to_webviews(
                SeelenEvent::WidgetPermissionsChanged,
                WIDGET_PERMISSIONS.get_all(),
            );
        });
    });
    &WIDGET_PERMISSIONS
}

impl crate::tauri_handlers::Handlers {
    pub fn get_widget_permissions() -> WidgetPermissions {
        get_manager().get_all()
    }

    /// Writing is not a grantable permission: only the settings app can edit the
    /// decisions, so no widget can escalate its own access.
    pub fn set_widget_permissions(
        webview: tauri::WebviewWindow,
        permissions: WidgetPermissions,
    ) -> Result<()> {
        let label = WidgetWebviewLabel::try_from_raw(webview.label())
            .map_err(|_| "Permission denied: caller is not a widget webview.")?;
        if label.widget_id != WidgetId::known_settings() {
            return Err(
                "Permission denied: only the settings app can edit widget permissions.".into(),
            );
        }
        get_manager().set_all(permissions)
    }
}

/* impl crate::tauri_handlers::Handlers {
    /// Dev-only command: simulates a permission request for any widget ID and perm.
    /// Follows the same flow as a real request (checks cache, shows dialog, persists result).
    pub fn simulate_perm(widget_id: String, perm: WidgetPerm) -> Result<()> {
        WIDGET_PERMISSIONS.request(&WidgetId::from(widget_id.as_str()), perm)
    }
}
 */
