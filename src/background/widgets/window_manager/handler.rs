use std::{collections::HashMap, sync::LazyLock};

use positioning::{easings::Easing, AppWinAnimation};
use windows::Win32::{Foundation::RECT, UI::WindowsAndMessaging::SW_NORMAL};

use crate::{
    error::Result,
    state::application::{performance::PERFORMANCE_MODE, FULL_STATE},
    windows_api::{window::Window, WindowsApi},
};
use seelen_core::{rect::Rect, state::PerformanceMode};

static ANIMATION_INSTANCE: LazyLock<tokio::sync::Mutex<Option<AppWinAnimation>>> =
    LazyLock::new(|| tokio::sync::Mutex::new(None));

#[tauri::command(async)]
pub async fn set_app_windows_positions(positions: HashMap<isize, Rect>) -> Result<()> {
    let mut positioner = positioning::Positioner::new();

    for (hwnd, rect) in positions {
        let window = Window::from(hwnd);
        if !window.is_window() || window.is_minimized() {
            continue;
        }

        if window.is_maximized() {
            window.show_window(SW_NORMAL)?; // unmaximize
        }

        let shadow = WindowsApi::shadow_rect(window.hwnd())?;
        let desired_rect = RECT {
            top: rect.top + shadow.top,
            left: rect.left + shadow.left,
            right: rect.right + shadow.right,
            bottom: rect.bottom + shadow.bottom,
        };

        positioner.add(hwnd, desired_rect.into());
    }

    // the guards avoid playing multiple animations at the same time.
    let mut guard = ANIMATION_INSTANCE.lock().await;
    if let Some(mut last) = guard.take() {
        last.interrupt();
        last.wait();
    }

    let state = FULL_STATE.load();
    let perf_mode = PERFORMANCE_MODE.load();

    if !state.settings.by_widget.wm.animations.enabled || **perf_mode != PerformanceMode::Disabled {
        positioner.place()?;
        return Ok(());
    }

    let duration = state.settings.by_widget.wm.animations.duration_ms;
    let easing = Easing::from_name(&state.settings.by_widget.wm.animations.ease_function)
        .unwrap_or(Easing::Linear);
    *guard = Some(positioner.place_animated(duration, easing, |result| {
        if let Err(err) = result {
            log::error!("Animated window placement failed: {err}");
        }
    })?);
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
