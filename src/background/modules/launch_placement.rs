use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use windows::Win32::{
    Foundation::RECT,
    UI::WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOZORDER},
};

use crate::{
    error::{Result, ResultLogExt},
    modules::apps::application::{UserAppWinEvent, UserAppsManager},
    utils::lock_free::SyncHashMap,
    windows_api::window::Window,
};

/// Windows decides where a newly launched app's window appears (usually wherever
/// it last closed, or the primary monitor) with no regard for where the user
/// triggered the launch from. When launched via Seelen (apps-menu, dock/weg),
/// remember the monitor under the cursor at launch time and, once the process'
/// first window shows up, move it there.
const PENDING_TTL: Duration = Duration::from_secs(20);

static PENDING: LazyLock<SyncHashMap<u32, (RECT, Instant)>> = LazyLock::new(SyncHashMap::new);

pub fn init() {
    UserAppsManager::subscribe(|event| {
        if let UserAppWinEvent::Added(addr) = event {
            place_on_target_monitor(&Window::from(addr)).log_error();
        }
    });
}

/// Call right after launching a process from a Seelen-triggered action, with
/// the monitor the user's cursor was on at that moment.
pub fn register(pid: u32, target_monitor_rect: RECT) {
    PENDING.retain(|(_, (_, at))| at.elapsed() < PENDING_TTL);
    PENDING.upsert(pid, (target_monitor_rect, Instant::now()));
}

fn place_on_target_monitor(window: &Window) -> Result<()> {
    let pid = window.process().id();
    let Some((target, registered_at)) = PENDING.get(&pid, |v| *v) else {
        return Ok(());
    };
    if registered_at.elapsed() > PENDING_TTL {
        PENDING.remove(&pid);
        return Ok(());
    }

    if !window.is_interactable_and_not_hidden() || window.is_maximized() || window.is_minimized() {
        return Ok(());
    }

    let current = window.monitor().info()?.monitorInfo.rcMonitor;
    if current.left == target.left && current.top == target.top {
        return Ok(()); // already on the right monitor
    }

    // Preserve the window's relative position/size, just move it to the target monitor.
    let dx = target.left - current.left;
    let dy = target.top - current.top;
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
