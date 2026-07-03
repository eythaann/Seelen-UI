use std::{
    sync::LazyLock,
    time::{Duration, Instant},
};

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, RECT},
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    },
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
///
/// Generous TTL: the launched process may show a "can't verify publisher"
/// prompt (common for network-drive exes) that waits on the user, or run
/// through antivirus/SmartScreen scanning, before its real window appears.
const PENDING_TTL: Duration = Duration::from_secs(60);
/// Many launchers (game boot/updater screens, installers) spawn the real app
/// as a brand new process instead of just opening a window themselves. Walk
/// up the parent chain this many hops looking for a pending launch.
const MAX_ANCESTOR_HOPS: u8 = 8;

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
    log::trace!("launch_placement: registering pid={pid} target={target_monitor_rect:?}");
    PENDING.retain(|(_, (_, at))| at.elapsed() < PENDING_TTL);
    PENDING.upsert(pid, (target_monitor_rect, Instant::now()));
}

fn place_on_target_monitor(window: &Window) -> Result<()> {
    let pid = window.process().id();
    let Some((target, _)) = find_pending(pid) else {
        return Ok(());
    };
    log::trace!(
        "launch_placement: matched new window (hwnd={:?}, pid={pid}) to target={target:?}",
        window.hwnd()
    );

    // Found via an ancestor: remember the real pid too, so sibling/later
    // windows from this same process match directly without re-walking.
    PENDING.get_or_insert(pid, || (target, Instant::now()), |_| {});

    if !window.is_interactable_and_not_hidden() || window.is_maximized() || window.is_minimized() {
        log::trace!(
            "launch_placement: skipping pid={pid}, interactable={} maximized={} minimized={}",
            window.is_interactable_and_not_hidden(),
            window.is_maximized(),
            window.is_minimized()
        );
        return Ok(());
    }

    let current = window.monitor().info()?.monitorInfo.rcMonitor;
    if current.left == target.left && current.top == target.top {
        log::trace!("launch_placement: pid={pid} already on target monitor");
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
    log::trace!("launch_placement: moving pid={pid} to {new_rect:?}");
    window.set_position(
        &new_rect,
        SWP_NOZORDER | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS,
    )
}

/// Looks up `pid` directly, then its ancestor chain, for a live pending entry.
fn find_pending(pid: u32) -> Option<(RECT, Instant)> {
    if PENDING.is_empty() {
        return None; // common case: nothing was launched via Seelen recently
    }

    let mut chain = vec![pid];
    let mut current = pid;
    for _ in 0..=MAX_ANCESTOR_HOPS {
        if let Some(entry) = PENDING.get(&current, |v| *v) {
            if entry.1.elapsed() <= PENDING_TTL {
                return Some(entry);
            }
            PENDING.remove(&current);
            return None;
        }
        current = match parent_process_id(current) {
            Some(ppid) if ppid != 0 && ppid != current => {
                chain.push(ppid);
                ppid
            }
            _ => break,
        };
    }
    log::trace!("launch_placement: no pending match for pid chain {chain:?}");
    None
}

fn parent_process_id(pid: u32) -> Option<u32> {
    // SAFETY: standard CreateToolhelp32Snapshot/Process32*W walk; handle is closed below.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let result = walk_for_parent(snapshot, pid);
        let _ = CloseHandle(snapshot);
        result
    }
}

unsafe fn walk_for_parent(snapshot: HANDLE, pid: u32) -> Option<u32> {
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    if Process32FirstW(snapshot, &mut entry).is_err() {
        return None;
    }
    loop {
        if entry.th32ProcessID == pid {
            return Some(entry.th32ParentProcessID);
        }
        if Process32NextW(snapshot, &mut entry).is_err() {
            return None;
        }
    }
}
