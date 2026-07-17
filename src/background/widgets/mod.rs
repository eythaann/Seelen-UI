pub mod cli;
pub mod loader;
pub mod manager;
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
    sync::LazyLock,
};

use seelen_core::{
    handlers::SeelenEvent,
    resource::ResourceId,
    state::{
        context_menu::ContextMenu, dialog::Dialog, Alignment, WidgetDebugInfo, WidgetInstanceMode,
        WidgetStatus, WidgetTriggerPayload,
    },
    system_state::{AppBarEdge, ZOrder},
    Rect,
};
use tauri::{Emitter, Manager};
use windows::Win32::Foundation::{HWND, RECT};

use crate::{
    app::{emit_to_webviews, get_app_handle},
    error::Result,
    resources::RESOURCES,
    state::application::FULL_STATE,
    utils::{atomic_write_file, constants::SEELEN_COMMON, lock_free::SyncHashMap},
    widgets::{manager::WIDGET_MANAGER, webview::WidgetWebviewLabel},
    windows_api::{
        input::{Keyboard, Mouse},
        AppBarData, WindowsApi,
    },
};

static PENDING_TRIGGERS: LazyLock<SyncHashMap<WidgetWebviewLabel, WidgetTriggerPayload>> =
    LazyLock::new(SyncHashMap::new);

#[tauri::command(async)]
pub fn set_current_widget_status(
    webview: tauri::WebviewWindow,
    status: WidgetStatus,
) -> Result<()> {
    let label = WidgetWebviewLabel::try_from_raw(webview.label())?;
    WIDGET_MANAGER.set_status(&label, status);

    if let Some(pending) = PENDING_TRIGGERS.remove(&label) {
        log::info!("Emitting pending trigger for {label}");
        get_app_handle().emit_to(label.raw, SeelenEvent::WidgetTriggered, &pending)?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn trigger_widget(payload: WidgetTriggerPayload) -> Result<()> {
    trigger_widget_inner(payload, None)
}

fn trigger_widget_inner(
    mut payload: WidgetTriggerPayload,
    owner_hwnd: Option<isize>,
) -> Result<()> {
    let state = FULL_STATE.load();
    if !state.is_widget_enabled(&payload.id) {
        return Err("Can't trigger a disabled widget".into());
    }

    let widget = RESOURCES
        .widgets
        .get(&payload.id)
        .ok_or("Widget not found")?
        .clone();

    let monitor_id = payload.monitor_id.as_ref().map(|id| id.to_string());
    let label = WidgetWebviewLabel::new(
        &payload.id,
        monitor_id.as_deref(),
        payload.instance_id.as_ref(),
    );

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

    if payload.desired_position.is_none() {
        payload.desired_position = Some(Mouse::get_cursor_pos()?);
    }

    if !WIDGET_MANAGER.is_ready(&label) {
        log::trace!("Trigger postponed, because widget instance is not ready: {label}");
        PENDING_TRIGGERS.upsert(label.clone(), payload);

        WIDGET_MANAGER.deployments.get(&label.widget_id, |c| {
            c.start_webview(&label);
        });
        return Ok(());
    }

    get_app_handle().emit_to(label.raw, SeelenEvent::WidgetTriggered, &payload)?;
    Ok(())
}

#[tauri::command(async)]
pub fn trigger_context_menu(
    webview: tauri::WebviewWindow,
    menu: ContextMenu,
    forward_to: Option<String>,
) -> Result<()> {
    let owner = WidgetWebviewLabel::try_from_raw(webview.label())?;
    let owner_hwnd = webview.hwnd()?.0 as isize;

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
#[tauri::command(async)]
pub fn trigger_dialog(dialog: Dialog, webview: tauri::WebviewWindow) -> Result<()> {
    let owner = WidgetWebviewLabel::try_from_raw(webview.label())?;
    let owner_hwnd = webview.hwnd()?.0 as isize;

    let mut payload = WidgetTriggerPayload::new("@seelen/dialog".into());
    payload.instance_id = Some(dialog.identifier);

    payload.add_custom_arg("dialog", serde_json::to_value(&dialog)?);
    payload.add_custom_arg("owner", serde_json::to_value(&owner.raw)?);

    trigger_widget_inner(payload, Some(owner_hwnd))
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

#[tauri::command(async)]
pub fn get_self_window_handle(webview: tauri::WebviewWindow) -> Result<isize> {
    Ok(webview.hwnd()?.0 as isize)
}

#[tauri::command(async)]
pub fn set_self_position(webview: tauri::WebviewWindow, rect: Rect) -> Result<()> {
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::UI::WindowsAndMessaging::SWP_ASYNCWINDOWPOS;

    let hwnd = HWND(webview.hwnd()?.0);
    let rect = RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };

    // pre set position for resize in case of multiples dpi
    WindowsApi::move_window(hwnd, &rect)?;
    WindowsApi::set_position(hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;
    // ensure child windows are redrawn
    unsafe {
        let _ = RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN | RDW_FRAME | RDW_ERASE,
        );
    }
    Ok(())
}

#[tauri::command(async)]
pub fn set_self_z_order(webview: tauri::WebviewWindow, z_order: ZOrder) -> Result<()> {
    use windows::Win32::UI::WindowsAndMessaging::{
        HWND_BOTTOM, HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST,
    };
    let hwnd = HWND(webview.hwnd()?.0);

    WindowsApi::set_z_order(
        hwnd,
        match z_order {
            ZOrder::TopMost => HWND_TOPMOST,
            ZOrder::NoTopMost => HWND_NOTOPMOST,
            ZOrder::Top => HWND_TOP,
            ZOrder::Bottom => HWND_BOTTOM,
        },
    )?;

    Ok(())
}

pub fn show_settings() -> Result<()> {
    trigger_widget(WidgetTriggerPayload::new("@seelen/settings".into()))
}

pub fn show_settings_at(route: &str) -> Result<()> {
    let mut payload = WidgetTriggerPayload::new("@seelen/settings".into());
    payload.add_custom_arg("route", route);
    trigger_widget(payload)
}

#[tauri::command(async)]
pub fn show_start_menu() -> Result<()> {
    let guard = FULL_STATE.load();
    if guard.is_widget_enabled(&"@seelen/apps-menu".into()) {
        trigger_widget(WidgetTriggerPayload::new("@seelen/apps-menu".into()))?;
        return Ok(());
    }
    // trick for showing the native start menu
    Keyboard::new().send_keys("{win}")
}

#[tauri::command(async)]
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

#[tauri::command(async)]
pub fn read_data_file(webview: tauri::WebviewWindow, filename: String) -> Result<String> {
    let base_path = widget_data_dir(&webview)?;
    let path = resolve_safe_path(&base_path, &filename)?;
    Ok(std::fs::read_to_string(path)?)
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

#[tauri::command(async)]
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

#[tauri::command(async)]
pub fn debug_open_dev_tools(label: String) -> Result<()> {
    let window = get_app_handle()
        .get_webview_window(&label)
        .ok_or("Widget window not found")?;
    window.open_devtools();
    Ok(())
}

#[tauri::command(async)]
pub fn register_app_bar(webview: tauri::WebviewWindow, rect: Rect, edge: AppBarEdge) -> Result<()> {
    let label = WidgetWebviewLabel::try_from_raw(webview.label())?;
    log::info!(target: &label.decoded, "Registering as Shell Bar");

    let hwnd = HWND(webview.hwnd()?.0);
    let rect = RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    };
    let mut app_bar = AppBarData::from_handle(hwnd);
    app_bar.set_rect(rect);
    app_bar.set_edge(edge);
    app_bar.register_as_new_bar()?;
    Ok(())
}

#[tauri::command(async)]
pub fn unregister_app_bar(webview: tauri::WebviewWindow) -> Result<()> {
    let label = WidgetWebviewLabel::try_from_raw(webview.label())?;
    log::info!(target: &label.decoded, "Unregistering as Shell Bar");

    let hwnd = HWND(webview.hwnd()?.0);
    let mut app_bar = AppBarData::from_handle(hwnd);
    app_bar.unregister_bar()?;
    Ok(())
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
