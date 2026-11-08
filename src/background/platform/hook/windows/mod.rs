mod event;

use event::WinEvent;

use std::{collections::HashSet, sync::Arc};

use parking_lot::RwLock;
use seelen_core::handlers::SeelenEvent;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook},
        WindowsAndMessaging::{
            DispatchMessageW, EVENT_MAX, EVENT_MIN, GetMessageW, MSG, OBJID_WINDOW,
            TranslateMessage,
        },
    },
};

use crate::{
    app::emit_to_webviews, error::ResultLogExt, platform::window::Window, state::FULL_STATE,
    utils::spawn_named_thread, widgets::weg::SeelenWeg, windows_api::WindowEnumerator,
};

use super::{HookManager, WindowEvent};

pub fn start() {
    init_zombie_window_killer();
    start_win_event_hook();

    let eid = HookManager::subscribe(|(event, origin)| match event {
        WindowEvent::MoveSizeStarted => {
            origin.set_dragging(true);
        }
        WindowEvent::MoveSizeEnded => {
            origin.set_dragging(false);
        }
        _ => (),
    });
    HookManager::set_event_handler_priority(&eid, 3);
}

/// this will be called without waiting for the event to be processed
/// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook#remarks
extern "system" fn raw_win_event_hook_recv(
    _hook_handle: HWINEVENTHOOK,
    event: u32,
    origin: HWND,
    id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if id_object != OBJID_WINDOW.0 {
        return;
    }

    // CRITICAL: Skip event processing when session is not interactive (locked/switched)
    // This prevents excessive CPU usage from processing thousands of events per second
    // when the user has locked the screen or switched users
    if !crate::platform::session::is_interactive() {
        return;
    }

    // deprecated code refactor this on future.
    if FULL_STATE.load().is_weg_enabled() {
        // raw events should be only used for a fastest and immediately processing
        SeelenWeg::process_raw_win_event(event, origin).log_error();
    }

    let event = WinEvent::from(event);
    let origin = Window::from(origin);
    if let Some(window_event) = event.to_window_event() {
        HookManager::send((window_event, origin));
    }
    event.debounce_as_needed(&origin);
}

fn start_win_event_hook() {
    // The interactive session check here is not redundant with the one in
    // raw_win_event_hook_recv. On a session switch, Windows fires events for ALL
    // session windows simultaneously, flooding the unbounded channel before
    // WM_WTSSESSION_CHANGE can be delivered to BgWindowProc (which runs on a
    // separate thread). The dispatcher thread then drains that backlog calling
    // expensive Win32 APIs (focused_app_snapshot, etc.) for every queued
    // event even though the session is already not interactive. This check makes
    // each backlogged event a cheap no-op (atomic load + branch) instead.
    let eid = HookManager::subscribe(|(event, mut origin)| {
        if !crate::platform::session::is_interactive() {
            return;
        }
        if event == WindowEvent::Foregrounded {
            origin = Window::get_foregrounded(); // sometimes this event is emited with the wrong origin
        }
        legacy_process_event(event, origin);
    });
    HookManager::set_event_handler_priority(&eid, 1);

    spawn_named_thread("WinEventHook", move || unsafe {
        SetWinEventHook(
            EVENT_MIN,
            EVENT_MAX,
            None,
            Some(raw_win_event_hook_recv),
            0,
            0,
            0,
        );

        let mut msg: MSG = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

#[allow(dead_code)]
fn log_event(event: WindowEvent, origin: Window) {
    use owo_colors::OwoColorize;
    let event_value = event.green();
    if event == WindowEvent::Destroyed {
        return println!("{event_value:?}({:0x})", origin.address());
    }
    println!("{event_value:?} | {origin:?}");
}

fn legacy_process_event(event: WindowEvent, origin: Window) {
    // log_event(event, origin);

    // TODO: optimize this, update independent fields instead of the whole struct
    {
        let shoup_update_focused = matches!(
            event,
            WindowEvent::Foregrounded
                | WindowEvent::ForegroundRectChanged
                | WindowEvent::TitleChanged
                | WindowEvent::MoveSizeStarted
                | WindowEvent::MoveSizeEnded
        );

        if shoup_update_focused && origin.is_focused() {
            emit_to_webviews(
                SeelenEvent::GlobalFocusChanged,
                origin.focused_app_snapshot(),
            );
        }
    }
}

fn init_zombie_window_killer() {
    let existing_windows = Arc::new(RwLock::new(HashSet::new()));

    let dict = existing_windows.clone();
    HookManager::subscribe(move |(event, origin)| match event {
        WindowEvent::Created => {
            dict.write().insert(origin.address());
        }
        WindowEvent::Destroyed => {
            dict.write().remove(&origin.address());
        }
        _ => {}
    });

    // Spawns a task that periodically checks for "zombie windows" - windows
    // that have been destroyed (e.g., through task kill or abnormal termination) but didn't
    // properly emit the ObjectDestroy event. This thread detects such windows
    // and emits the missing destruction events to ensure proper cleanup.
    spawn_named_thread("Zombie Window Exterminator", move || {
        WindowEnumerator::new()
            .for_each_and_descendants(|hwnd| {
                existing_windows.write().insert(hwnd.0 as isize);
            })
            .log_error();

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));

            // Pause when session is not interactive to reduce CPU usage
            if !crate::platform::session::is_interactive() {
                continue;
            }

            let dead: Vec<isize> = {
                let guard = existing_windows.read();
                guard
                    .iter()
                    .copied()
                    .filter(|&addr| !Window::from(addr).is_window())
                    .collect()
            };

            for addr in dead {
                existing_windows.write().remove(&addr);
                let window = Window::from(addr);
                log::trace!("Reaping window: {:0x}", window.address());
                HookManager::send((WindowEvent::Destroyed, window));
            }
        }
    });
}
