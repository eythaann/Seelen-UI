use std::{
    collections::HashMap,
    sync::{LazyLock, Once},
};

use slu_ipc::messages::SvcAction;
use windows::Win32::UI::WindowsAndMessaging::SW_NORMAL;

use crate::{
    app::emit_to_webviews,
    cli::ServicePipe,
    error::Result,
    state::application::{performance::PERFORMANCE_MODE, FULL_STATE},
    widgets::window_manager::state_v2::{TwmState, WM_STATE},
    windows_api::{window::Window, WindowsApi},
};
use seelen_core::{
    handlers::SeelenEvent,
    rect::Rect,
    state::{PerformanceMode, TwmGlobalRuntimeTree},
};

#[tauri::command(async)]
pub fn wm_get_render_tree() -> TwmGlobalRuntimeTree {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        TwmState::subscribe(|_event| {
            let guard = WM_STATE.lock();
            guard.restore_stacks();
            emit_to_webviews(SeelenEvent::WMTreeChanged, &guard.state);
        });
    });

    WM_STATE.lock().state.clone()
}

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
        if !window.is_window() || window.is_minimized() {
            continue;
        }

        // avoid to move window while dragging
        if window.is_dragging() {
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

    // Update node.rect for tiled windows based on the computed layout positions.
    {
        let mut state = WM_STATE.lock();
        for (hwnd, rect) in &list {
            state.set_cached_node_rect(*hwnd, rect.clone());
        }
    }

    ServicePipe::request(SvcAction::DeferWindowPositions {
        list,
        animated: place_animated,
        animation_duration: state.settings.by_widget.wm.animations.duration_ms,
        easing: state.settings.by_widget.wm.animations.ease_function.clone(),
    })?;
    Ok(())
}

/// TODO delete this is used only by webview, but this should use self_focus command.
#[tauri::command(async)]
pub fn request_focus(hwnd: isize) -> Result<()> {
    let window = Window::from(hwnd);
    if !window.is_window() {
        return Ok(());
    }
    window.focus()?;
    Ok(())
}
