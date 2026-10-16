use seelen_core::{
    resource::WidgetId,
    state::{FancyToolbarSide, HideMode},
};
use windows::Win32::UI::WindowsAndMessaging::{WS_CHILD, WS_EX_TOPMOST};
use windows::Win32::{Foundation::RECT, UI::WindowsAndMessaging::SWP_ASYNCWINDOWPOS};

use crate::{
    error::Result,
    state::application::FULL_STATE,
    widgets::{toolbar::FancyToolbar, window_manager::WindowManagerV2},
    windows_api::window::{Window, event::WinEvent},
};

/// Keeps unmanaged windows below a permanently visible top toolbar.
///
/// The window manager intentionally leaves some windows alone (for example, games and fixed-size
/// windows). Those windows still need to make room for the toolbar when they place themselves at
/// the top edge of a monitor.
pub fn process_win_event(event: WinEvent, mut window: Window) -> Result<()> {
    if !matches!(
        event,
        WinEvent::SystemForeground | WinEvent::SystemMoveSizeEnd | WinEvent::SynDebouncedRectChange
    ) {
        return Ok(());
    }

    if event == WinEvent::SystemForeground {
        // The accessibility hook can occasionally report the wrong origin for foreground events.
        window = Window::get_foregrounded();
    }

    if !window.is_window()
        || !window.is_visible()
        || window.is_minimized()
        || window.is_fullscreen()
        || window.process().is_seelen()
        || crate::windows_api::WindowsApi::get_styles(window.hwnd()).contains(WS_CHILD)
        || crate::windows_api::WindowsApi::get_ex_styles(window.hwnd()).contains(WS_EX_TOPMOST)
    {
        return Ok(());
    }

    // Location events are emitted while a user is dragging. Wait for the move-size end event so
    // the window is adjusted once, after the user's move is complete.
    if event != WinEvent::SystemMoveSizeEnd && window.is_dragging() {
        return Ok(());
    }

    // Windows that would normally be tiled are left to the window manager. This targets only
    // windows explicitly or implicitly excluded from tiling.
    if WindowManagerV2::should_be_managed(window.hwnd()) {
        return Ok(());
    }

    let monitor = window.monitor();
    let monitor_id = monitor.stable_id()?;
    let state = FULL_STATE.load();
    let toolbar = &state.settings.by_widget.fancy_toolbar;
    if !state
        .settings
        .is_widget_enabled_on_monitor(&WidgetId::known_toolbar(), &monitor_id)
        || toolbar.position != FancyToolbarSide::Top
        || toolbar.hide_mode != HideMode::Never
    {
        return Ok(());
    }

    let monitor_rect = monitor.rect()?;
    let toolbar_bottom = monitor_rect.top + FancyToolbar::get_toolbar_height_on_monitor(&monitor)?;
    let current = window.outer_rect()?;
    if current.top >= toolbar_bottom {
        return Ok(());
    }

    let offset = toolbar_bottom - current.top;
    let target = RECT {
        left: current.left,
        top: current.top + offset,
        right: current.right,
        bottom: current.bottom + offset,
    };
    window.set_position(&target, SWP_ASYNCWINDOWPOS)?;
    Ok(())
}
