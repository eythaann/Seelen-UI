use std::collections::HashMap;

use slu_ipc::messages::SvcAction;
use windows::Win32::UI::WindowsAndMessaging::SW_NORMAL;

use crate::{
    cli::ServicePipe,
    error::Result,
    state::application::{performance::PERFORMANCE_MODE, FULL_STATE},
    windows_api::{window::Window, WindowsApi},
};
use seelen_core::{rect::Rect, state::PerformanceMode};

#[tauri::command(async)]
pub fn set_app_windows_positions(positions: HashMap<isize, Rect>) -> Result<()> {
    let mut list = HashMap::new();

    // map and filter step
    for (hwnd, rect) in positions {
        let window = Window::from(hwnd);
        if !window.is_window() || window.is_minimized() {
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
