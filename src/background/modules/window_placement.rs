use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOZORDER};

use crate::{
    error::{Result, ResultLogExt},
    modules::apps::application::{UserAppWinEvent, UserAppsManager},
    windows_api::window::Window,
};

/// Keep newly-opened windows out of the space reserved by shell bars (e.g. the
/// top toolbar). Some apps open a normal-sized window at the very top of the
/// monitor, ignoring the reserved work area, which leaves their title bar and
/// caption buttons hidden behind the toolbar. Maximized windows already respect
/// the work area, so this only nudges restored windows that intrude.
pub fn init() {
    UserAppsManager::subscribe(|event| {
        if let UserAppWinEvent::Added(addr) = event {
            nudge_into_work_area(&Window::from(addr)).log_error();
        }
    });
}

fn nudge_into_work_area(window: &Window) -> Result<()> {
    if !window.is_interactable_and_not_hidden()
        || window.is_maximized()
        || window.is_minimized()
        || window.is_fullscreen()
    {
        return Ok(());
    }

    // rcWork already excludes any registered app bars (toolbar/dock).
    let work = window.monitor().info()?.monitorInfo.rcWork;
    let inner = window.inner_rect()?;

    // How far the real (shadow-less) frame intrudes past the reserved edges.
    let dx = (work.left - inner.left).max(0);
    let dy = (work.top - inner.top).max(0);
    if dx == 0 && dy == 0 {
        return Ok(());
    }

    // Translate the whole window by the same delta (keeps its size).
    let outer = window.outer_rect()?;
    let new_rect = RECT {
        left: outer.left + dx,
        top: outer.top + dy,
        right: outer.right + dx,
        bottom: outer.bottom + dy,
    };
    window.set_position(
        &new_rect,
        SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS,
    )
}
