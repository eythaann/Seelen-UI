use std::{
    collections::HashMap,
    sync::{LazyLock, Once},
};

use slu_ipc::messages::SvcAction;
use tauri::Emitter;
use windows::Win32::UI::WindowsAndMessaging::SW_NORMAL;

use crate::{
    app::get_app_handle,
    cli::ServicePipe,
    error::{Result, ResultLogExt},
    state::application::{performance::PERFORMANCE_MODE, FULL_STATE},
    widgets::window_manager::state::{WmState, WM_STATE},
    windows_api::{window::Window, WindowsApi},
};
use seelen_core::{
    handlers::SeelenEvent,
    rect::Rect,
    state::{PerformanceMode, WmRenderTree},
};

static SCHEDULED_POSITIONS: LazyLock<scc::HashMap<isize, Rect>> = LazyLock::new(scc::HashMap::new);

/// will schedule the position to be sent in a batch on the next window manager update
pub fn schedule_window_position(window: isize, rect: Rect) {
    SCHEDULED_POSITIONS.upsert(window, rect);
}

#[tauri::command(async)]
pub fn set_app_windows_positions(positions: HashMap<isize, Rect>) -> Result<()> {
    let mut list = HashMap::new();

    // map and filter step
    for (hwnd, rect) in positions {
        let window = Window::from(hwnd);
        if !window.is_window() || window.is_minimized() || window.is_dragging() {
            continue;
        }

        if window.is_maximized() {
            window.show_window(SW_NORMAL)?; // unmaximize
        }

        let shadow = WindowsApi::shadow_rect(window.hwnd())?;
        let desired_rect = Rect {
            top: rect.top + shadow.top,
            left: rect.left + shadow.left,
            right: rect.right + shadow.right,
            bottom: rect.bottom + shadow.bottom,
        };
        list.insert(hwnd, desired_rect);
    }

    SCHEDULED_POSITIONS.scan(|k, v| {
        let window = Window::from(*k);
        if window.is_window() && !window.is_minimized() && !window.is_dragging() {
            list.insert(*k, v.clone());
        }
    });
    SCHEDULED_POSITIONS.clear();

    let state = FULL_STATE.load();
    let perf_mode = PERFORMANCE_MODE.load();
    let place_animated =
        state.settings.by_widget.wm.animations.enabled && **perf_mode == PerformanceMode::Disabled;

    ServicePipe::request(SvcAction::DeferWindowPositions {
        list,
        animated: place_animated,
        animation_duration: state.settings.by_widget.wm.animations.duration_ms,
        easing: state.settings.by_widget.wm.animations.ease_function.clone(),
    })?;
    Ok(())
}

#[tauri::command(async)]
pub fn request_focus(hwnd: isize) -> Result<()> {
    let window = Window::from(hwnd);
    if !window.is_window() {
        return Ok(());
    }
    window.focus()?;
    Ok(())
}

#[tauri::command(async)]
pub fn wm_get_render_tree() -> WmRenderTree {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        WmState::subscribe(|_event| {
            get_app_handle()
                .emit(
                    SeelenEvent::WMTreeChanged,
                    &WM_STATE.lock().get_render_tree(),
                )
                .log_error();
        });
    });

    WM_STATE.lock().get_render_tree()
}
