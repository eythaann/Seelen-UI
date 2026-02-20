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
    state::{context_menu::ContextMenu, WidgetInstanceMode, WidgetStatus, WidgetTriggerPayload},
    Rect,
};
use tauri::Emitter;

use crate::{
    app::get_app_handle,
    error::Result,
    resources::RESOURCES,
    state::application::FULL_STATE,
    utils::{constants::SEELEN_COMMON, lock_free::SyncHashMap},
    widgets::{manager::WIDGET_MANAGER, webview::WidgetWebviewLabel},
    windows_api::{
        input::{Keyboard, Mouse},
        WindowsApi,
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
        get_app_handle().emit_to(label.raw, SeelenEvent::WidgetTriggered, &pending)?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn trigger_widget(mut payload: WidgetTriggerPayload) -> Result<()> {
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

            WIDGET_MANAGER.groups.get(&payload.id, |container| {
                if !container.instances.contains_key(&label) {
                    container.create_runtime_instance(instance_id);
                }
            });
        }
    }

    if payload.desired_position.is_none() {
        payload.desired_position = Some(Mouse::get_cursor_pos()?);
    }

    if !WIDGET_MANAGER.is_ready(&label) {
        log::warn!("Trigger postponed, because widget instance is not ready: {label}");
        PENDING_TRIGGERS.upsert(label.clone(), payload);

        WIDGET_MANAGER.groups.get(&label.widget_id, |c| {
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

    let mut payload = WidgetTriggerPayload::new("@seelen/context-menu".into());
    payload.instance_id = Some(menu.identifier);

    payload.add_custom_arg("menu", serde_json::to_value(menu)?);
    payload.add_custom_arg("owner", serde_json::to_value(&owner.raw)?);
    payload.add_custom_arg(
        "forwardTo",
        serde_json::to_value(forward_to.unwrap_or(owner.raw))?,
    );
    trigger_widget(payload)
}

#[tauri::command(async)]
pub fn get_self_window_handle(webview: tauri::WebviewWindow) -> Result<isize> {
    Ok(webview.hwnd()?.0 as isize)
}

#[tauri::command(async)]
pub fn set_self_position(webview: tauri::WebviewWindow, rect: Rect) -> Result<()> {
    use windows::Win32::Foundation::{HWND, RECT};
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

pub fn show_settings() -> Result<()> {
    trigger_widget(WidgetTriggerPayload::new("@seelen/settings".into()))
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
    std::fs::write(path, content)?;
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
