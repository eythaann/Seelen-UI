use std::{collections::HashMap, sync::Once};

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

/// Adds the shadow offset to `rect`, unmaximizing `window` first if needed. Returns `None` if
/// the window is currently in a state that shouldn't be repositioned (closed, minimized, or
/// being dragged by the user).
fn desired_rect_for(window: &Window, rect: &Rect) -> Result<Option<Rect>> {
    if !window.is_window() || window.is_minimized() || window.is_dragging() {
        return Ok(None);
    }

    if window.is_maximized() {
        window.show_window(SW_NORMAL)?; // unmaximize
    }

    let shadow = WindowsApi::shadow_rect(window.hwnd())?;
    Ok(Some(Rect {
        top: rect.top + shadow.top,
        left: rect.left + shadow.left,
        right: rect.right + shadow.right,
        bottom: rect.bottom + shadow.bottom,
    }))
}

#[tauri::command(async)]
pub fn set_app_windows_positions(positions: HashMap<isize, Rect>) -> Result<()> {
    log::trace!(
        "set_app_windows_positions called with {} positions",
        positions.len()
    );

    let mut list = HashMap::new();

    for (hwnd, rect) in &positions {
        let window = Window::from(*hwnd);
        if let Some(desired_rect) = desired_rect_for(&window, rect)? {
            list.insert(*hwnd, desired_rect);
        }
    }

    let state = FULL_STATE.load();
    let perf_mode = PERFORMANCE_MODE.load();
    let place_animated =
        state.settings.by_widget.wm.animations.enabled && perf_mode == PerformanceMode::Disabled;

    // Store inner (pre-shadow) rects — must match Window::inner_rect() used in comparisons.
    {
        let mut state = WM_STATE.lock();
        for hwnd in list.keys() {
            if let Some(rect) = positions.get(hwnd) {
                state.set_cached_node_rect(*hwnd, rect.clone());
            }
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

/// Immediately requests `window` be moved to `rect`, bypassing the batched layout-render path
/// used by `set_app_windows_positions`. Each window now animates independently on the service
/// side, so there is no longer a need to defer/batch a single ad-hoc reposition (e.g. floating
/// a window) until the next full layout render.
///
/// Does not touch `WM_STATE`, unlike `set_app_windows_positions` — this is called from places
/// that may already be holding the (non-reentrant) `WM_STATE` lock (e.g. `TwmState` methods).
pub fn set_app_window_position(window: &Window, rect: Rect) -> Result<()> {
    let Some(desired_rect) = desired_rect_for(window, &rect)? else {
        return Ok(());
    };

    let state = FULL_STATE.load();
    let perf_mode = PERFORMANCE_MODE.load();
    let place_animated =
        state.settings.by_widget.wm.animations.enabled && perf_mode == PerformanceMode::Disabled;

    let mut list = HashMap::new();
    list.insert(window.address(), desired_rect);

    ServicePipe::request(SvcAction::DeferWindowPositions {
        list,
        animated: place_animated,
        animation_duration: state.settings.by_widget.wm.animations.duration_ms,
        easing: state.settings.by_widget.wm.animations.ease_function.clone(),
    })?;
    Ok(())
}

#[tauri::command(async)]
pub fn wm_set_stack_active_window(hwnd: isize) -> Result<()> {
    let window = Window::from(hwnd);
    if !window.is_window() {
        return Ok(());
    }
    WM_STATE.lock().set_stack_active_window(&window)?;
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
