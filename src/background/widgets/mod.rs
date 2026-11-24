pub mod cli;
pub mod loader;
pub mod manager;
mod native_blur;
mod native_blur_effect;
pub mod permissions;
pub mod popups;
pub mod task_switcher;
pub mod toolbar;
pub mod wallpaper_manager;
pub mod webview;
pub mod weg;
pub mod window_manager;

use std::{
    path::{Path, PathBuf},
    sync::{
        LazyLock,
        atomic::{AtomicU64, Ordering},
    },
};

use seelen_core::{
    Rect,
    handlers::SeelenEvent,
    resource::ResourceId,
    state::{
        Alignment, WidgetDebugInfo, WidgetInstanceMode, WidgetPreset, WidgetStatus,
        WidgetTriggerPayload, context_menu::ContextMenu, dialog::Dialog,
    },
    system_state::{AppBarEdge, ZOrder},
};
use tauri::{Emitter, Manager};

use crate::{
    app::{emit_to_webviews, get_app_handle},
    error::{Result, ResultLogExt},
    platform::{cursor, shell},
    resources::RESOURCES,
    state::FULL_STATE,
    utils::{atomic_write_file, constants::SEELEN_COMMON, lock_free::SyncHashMap},
    widgets::{manager::WIDGET_MANAGER, webview::WidgetWebviewLabel},
};

/// A pending trigger and the toolbar popup switch generation it belongs to, if any.
type PendingTrigger = Option<(WidgetTriggerPayload, Option<u64>)>;

static PENDING_TRIGGERS: LazyLock<SyncHashMap<WidgetWebviewLabel, PendingTrigger>> =
    LazyLock::new(SyncHashMap::new);
static POPUP_SWITCH_GENERATION: AtomicU64 = AtomicU64::new(0);

pub fn native_blur_focus_changed(hwnd: isize, owner: isize) {
    native_blur::on_focus_changed(hwnd, owner);
}

fn popup_request_is_current(generation: Option<u64>) -> bool {
    generation.is_none_or(|value| value == POPUP_SWITCH_GENERATION.load(Ordering::Acquire))
}

fn trigger_widget_inner(payload: WidgetTriggerPayload, owner_hwnd: Option<isize>) -> Result<()> {
    trigger_widget_ordered(payload, owner_hwnd, None)
}

fn trigger_widget_ordered(
    mut payload: WidgetTriggerPayload,
    owner_hwnd: Option<isize>,
    generation: Option<u64>,
) -> Result<()> {
    if !popup_request_is_current(generation) {
        return Ok(());
    }
    let state = FULL_STATE.load();
    if !state.is_widget_enabled(&payload.id) {
        return Err("Can't trigger a disabled widget".into());
    }

    let widget = RESOURCES
        .widgets
        .read_sync(&payload.id, |_, w| w.clone())
        .ok_or("Widget not found")?;

    let monitor_id = payload.monitor_id.as_ref().map(|id| id.to_string());
    let label = WidgetWebviewLabel::new(
        &payload.id,
        monitor_id.as_deref(),
        payload.instance_id.as_ref(),
    );
    if widget.preset == WidgetPreset::Popup {
        native_blur::allow_popup_trigger(label.raw.clone(), generation);
    }

    match widget.instances {
        WidgetInstanceMode::Single => {}
        WidgetInstanceMode::ReplicaByMonitor => {
            if payload.monitor_id.is_none() {
                return Err("Monitor id is required for replica by monitor widgets".into());
            }
        }
        WidgetInstanceMode::Multiple => {
            let Some(instance_id) = &payload.instance_id else {
                return Err("Instance id is required for multiple instance widgets".into());
            };

            WIDGET_MANAGER.deployments.get(&payload.id, |container| {
                if !container.pods.contains_key(&label) {
                    container.create_runtime_instance(instance_id, owner_hwnd);
                }
            });
        }
    }

    match &mut payload.desired_position {
        None => payload.desired_position = Some(cursor::position()?),
        // `i32::MIN` marks an axis as unset, so it falls back to the cursor position on
        // that axis while keeping the caller-provided value on the other.
        Some(pos) if pos.x == i32::MIN || pos.y == i32::MIN => {
            let cursor = cursor::position()?;
            if pos.x == i32::MIN {
                pos.x = cursor.x;
            }
            if pos.y == i32::MIN {
                pos.y = cursor.y;
            }
        }
        Some(_) => {}
    }

    if !WIDGET_MANAGER.is_ready(&label) {
        if !popup_request_is_current(generation) {
            return Ok(());
        }
        log::trace!("Trigger postponed, because widget instance is not ready: {label}");
        // Check and replace under the same short pending-map lock: an older worker
        // cannot overwrite a newer request for the same still-mounting popup.
        let queued = PENDING_TRIGGERS.get_or_default(label.clone(), |pending| {
            if !popup_request_is_current(generation) {
                return false;
            }
            *pending = Some((payload, generation));
            true
        });
        if !queued {
            return Ok(());
        }

        WIDGET_MANAGER.deployments.get(&label.widget_id, |c| {
            c.start_webview(&label);
        });
        return Ok(());
    }

    if popup_request_is_current(generation) {
        get_app_handle().emit_to(label.raw, SeelenEvent::WidgetTriggered, &payload)?;
    }
    Ok(())
}

/// Trigger a dialog from backend code (no owner webview; button events are emitted globally).
pub fn trigger_dialog_backend(dialog: Dialog) -> Result<()> {
    let mut payload = WidgetTriggerPayload::new("@seelen/dialog".into());
    payload.instance_id = Some(dialog.identifier);
    payload.align_x = Some(Alignment::Center);
    payload.align_y = Some(Alignment::Center);

    payload.add_custom_arg("dialog", serde_json::to_value(&dialog)?);

    trigger_widget_inner(payload, None)
}

pub fn show_settings() -> Result<()> {
    crate::tauri_handlers::Handlers::trigger_widget(WidgetTriggerPayload::new(
        "@seelen/settings".into(),
    ))
}

pub fn show_settings_at(route: &str) -> Result<()> {
    let mut payload = WidgetTriggerPayload::new("@seelen/settings".into());
    payload.add_custom_arg("route", route);
    crate::tauri_handlers::Handlers::trigger_widget(payload)
}

fn widget_data_dir(webview: &tauri::WebviewWindow) -> Result<PathBuf> {
    let label = WidgetWebviewLabel::try_from_raw(webview.label())?;
    let data_dir = SEELEN_COMMON.app_data_dir().join("data");

    let folder = match &*label.widget_id {
        ResourceId::Local(id) => id.trim_start_matches("@").replace("/", "-"),
        ResourceId::Remote(uuid) => uuid.to_string(),
    };

    let path = data_dir.join(folder);
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn notify_widget_statuses_change() {
    std::thread::spawn(|| {
        let mut result = Vec::new();
        WIDGET_MANAGER.deployments.for_each(|(_, deployment)| {
            deployment.pods.for_each(|(_, pod)| {
                result.push(WidgetDebugInfo {
                    label: pod.label.raw.clone(),
                    widget_id: pod.label.widget_id.to_string(),
                    monitor_id: pod.label.monitor_id.as_ref().map(|m| m.to_string()),
                    instance_id: pod.label.instance_id.map(|id| id.to_string()),
                    status: *pod.status(),
                    webview_window_id: pod.hwnd(),
                });
            });
        });
        emit_to_webviews(SeelenEvent::WidgetDebugInfoChanged, result);
    });
}

fn resolve_safe_path(base: &Path, filename: &str) -> Result<PathBuf> {
    let filename_path = PathBuf::from(filename);

    if filename_path.is_absolute() {
        return Err("Absolute paths are not allowed".into());
    }

    let joined = base.join(filename_path);
    let base_canon = base.canonicalize()?;

    let target_canon = joined.canonicalize().or_else(|_| {
        // if file does not exist, canonicalize the parent
        let parent = joined.parent().ok_or("Invalid path")?;
        let parent_canon = parent.canonicalize()?;
        Result::Ok(parent_canon.join(joined.file_name().ok_or("Invalid filename")?))
    })?;

    if !target_canon.starts_with(&base_canon) {
        return Err("Path traversal attempt detected >:(".into());
    }

    Ok(target_canon)
}

impl crate::tauri_handlers::Handlers {
    pub fn set_current_widget_status(
        webview: tauri::WebviewWindow,
        status: WidgetStatus,
    ) -> Result<()> {
        let label = WidgetWebviewLabel::try_from_raw(webview.label())?;
        WIDGET_MANAGER.set_status(&label, status);

        if let Some(Some((pending, generation))) = PENDING_TRIGGERS.remove(&label)
            && popup_request_is_current(generation)
        {
            log::info!("Emitting pending trigger for {label}");
            get_app_handle().emit_to(label.raw, SeelenEvent::WidgetTriggered, &pending)?;
        }
        Ok(())
    }

    pub fn trigger_widget(payload: WidgetTriggerPayload) -> Result<()> {
        let toolbar = payload
            .custom_args
            .as_ref()
            .and_then(|args| args.get("__seelenToolbarToggle"))
            .is_some_and(|value| value.0.as_bool() == Some(true));
        let coordinates_switch = RESOURCES
            .widgets
            .read_sync(&payload.id, |_, widget| {
                widget.preset == WidgetPreset::Popup || widget.native_blur_switch == Some(true)
            })
            .unwrap_or(false);
        if toolbar && coordinates_switch {
            if !FULL_STATE.load().is_widget_enabled(&payload.id) {
                return Err("Can't trigger a disabled widget".into());
            }
            let generation = POPUP_SWITCH_GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
            crate::get_tokio_handle().spawn_blocking(move || {
                if !popup_request_is_current(Some(generation)) {
                    return;
                }
                let monitor = payload.monitor_id.as_ref().map(ToString::to_string);
                let label = WidgetWebviewLabel::new(
                    &payload.id,
                    monitor.as_deref(),
                    payload.instance_id.as_ref(),
                );
                let cold = !WIDGET_MANAGER.is_ready(&label);
                get_app_handle()
                    .run_on_main_thread(move || {
                        if !popup_request_is_current(Some(generation)) {
                            return;
                        }
                        native_blur::prepare_popup_switch(&payload.id);
                        let close = if cold {
                            native_blur::request_switch_close(&payload.id)
                        } else {
                            None
                        };
                        crate::get_tokio_handle().spawn(async move {
                            if let Some(close) = close {
                                // Release on native animation submission, or fall back if the outgoing
                                // popup cannot produce a path. No UI/deployment lock is held here.
                                let _ = tokio::time::timeout(
                                    std::time::Duration::from_millis(120),
                                    close,
                                )
                                .await;
                            }
                            crate::get_tokio_handle().spawn_blocking(move || {
                                trigger_widget_ordered(payload, None, Some(generation)).log_error();
                            });
                        });
                    })
                    .log_error();
            });
            return Ok(());
        }
        trigger_widget_inner(payload, None)
    }

    pub fn trigger_context_menu(
        webview: tauri::WebviewWindow,
        menu: ContextMenu,
        forward_to: Option<String>,
    ) -> Result<()> {
        let owner = WidgetWebviewLabel::try_from_raw(webview.label())?;
        let owner_hwnd = webview::native_window(&webview)?.address();

        let mut payload = WidgetTriggerPayload::new("@seelen/context-menu".into());
        payload.instance_id = Some(menu.identifier);
        payload.align_x = menu.align_x;
        payload.align_y = menu.align_y;

        payload.add_custom_arg("menu", serde_json::to_value(menu)?);
        payload.add_custom_arg("owner", serde_json::to_value(&owner.raw)?);
        payload.add_custom_arg(
            "forwardTo",
            serde_json::to_value(forward_to.unwrap_or(owner.raw))?,
        );
        trigger_widget_inner(payload, Some(owner_hwnd))
    }

    /// Trigger a dialog from within a widget (the owner webview is used for event routing).
    pub fn trigger_dialog(webview: tauri::WebviewWindow, dialog: Dialog) -> Result<()> {
        let owner = WidgetWebviewLabel::try_from_raw(webview.label())?;
        let owner_hwnd = webview::native_window(&webview)?.address();

        let mut payload = WidgetTriggerPayload::new("@seelen/dialog".into());
        payload.instance_id = Some(dialog.identifier);

        payload.add_custom_arg("dialog", serde_json::to_value(&dialog)?);
        payload.add_custom_arg("owner", serde_json::to_value(&owner.raw)?);

        trigger_widget_inner(payload, Some(owner_hwnd))
    }

    pub fn get_self_window_handle(webview: tauri::WebviewWindow) -> Result<isize> {
        Ok(webview::native_window(&webview)?.address())
    }

    pub fn normalize_self_device_pixel_ratio(webview: tauri::WebviewWindow) -> Result<()> {
        webview::lock_rasterization_scale_to_one(&webview)
    }

    pub fn set_self_position(webview: tauri::WebviewWindow, rect: Rect) -> Result<()> {
        webview::set_position(&webview, &rect)
    }

    pub fn set_self_z_order(webview: tauri::WebviewWindow, z_order: ZOrder) -> Result<()> {
        webview::native_window(&webview)?.set_z_order(z_order)
    }

    pub fn request_focus(webview: tauri::WebviewWindow) -> Result<()> {
        let window = webview::native_window(&webview)?;
        if !window.is_window() {
            return Ok(());
        }
        window.focus()?;
        Ok(())
    }

    pub fn show_start_menu() -> Result<()> {
        let guard = FULL_STATE.load();
        if guard.is_widget_enabled(&"@seelen/apps-menu".into()) {
            Self::trigger_widget(WidgetTriggerPayload::new("@seelen/apps-menu".into()))?;
            return Ok(());
        }
        shell::open_native_start_menu()
    }

    pub fn write_data_file(
        webview: tauri::WebviewWindow,
        filename: String,
        content: String,
    ) -> Result<()> {
        let base_path = widget_data_dir(&webview)?;
        let path = resolve_safe_path(&base_path, &filename)?;
        atomic_write_file(&path, content.as_bytes())?;
        Ok(())
    }

    pub fn read_data_file(webview: tauri::WebviewWindow, filename: String) -> Result<String> {
        let base_path = widget_data_dir(&webview)?;
        let path = resolve_safe_path(&base_path, &filename)?;
        Ok(std::fs::read_to_string(path)?)
    }

    pub fn debug_get_widgets_statuses() -> Vec<WidgetDebugInfo> {
        let mut result = Vec::new();
        WIDGET_MANAGER.deployments.for_each(|(_, deployment)| {
            deployment.pods.for_each(|(_, pod)| {
                result.push(WidgetDebugInfo {
                    label: pod.label.raw.clone(),
                    widget_id: pod.label.widget_id.to_string(),
                    monitor_id: pod.label.monitor_id.as_ref().map(|m| m.to_string()),
                    instance_id: pod.label.instance_id.map(|id| id.to_string()),
                    status: *pod.status(),
                    webview_window_id: pod.hwnd(),
                });
            });
        });
        result
    }

    pub fn debug_open_dev_tools(label: String) -> Result<()> {
        let window = get_app_handle()
            .get_webview_window(&label)
            .ok_or("Widget window not found")?;
        window.open_devtools();
        Ok(())
    }

    pub fn register_app_bar(
        webview: tauri::WebviewWindow,
        rect: Rect,
        edge: AppBarEdge,
    ) -> Result<()> {
        let label = WidgetWebviewLabel::try_from_raw(webview.label())?;
        log::info!(target: &label.decoded, "Registering as Shell Bar");

        webview::register_app_bar(&webview, &rect, edge)
    }

    pub fn unregister_app_bar(webview: tauri::WebviewWindow) -> Result<()> {
        let label = WidgetWebviewLabel::try_from_raw(webview.label())?;
        log::info!(target: &label.decoded, "Unregistering as Shell Bar");

        webview::unregister_app_bar(&webview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_switch_keeps_only_latest_toolbar_trigger_and_preserves_other_triggers() {
        let latest = POPUP_SWITCH_GENERATION.load(Ordering::Acquire);
        assert!(popup_request_is_current(Some(latest)));
        assert!(!popup_request_is_current(Some(latest.wrapping_add(1))));
        assert!(popup_request_is_current(None));
    }
}
