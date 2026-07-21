use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use parking_lot::RwLock;
use seelen_core::handlers::SeelenEvent;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, EVENT_MAX, EVENT_MIN, MSG,
            OBJID_WINDOW,
        },
    },
};

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    event_manager,
    state::application::FULL_STATE,
    utils::spawn_named_thread,
    widgets::weg::SeelenWeg,
    windows_api::{
        event_window::IS_INTERACTIVE_SESSION,
        input::Mouse,
        window::{event::WinEvent, Window},
        WindowEnumerator,
    },
};

pub static LOG_WIN_EVENTS: AtomicBool = AtomicBool::new(false);

pub struct HookManager;

event_manager!(HookManager, (WinEvent, Window));

impl HookManager {
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
        if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
            return;
        }

        // deprecated code refactor this on future.
        if FULL_STATE.load().is_weg_enabled() {
            // raw events should be only used for a fastest and immediately processing
            SeelenWeg::process_raw_win_event(event, origin).log_error();
        }

        let event = WinEvent::from(event);
        let origin = Window::from(origin);
        Self::send((event, origin));
        event.debounce_as_needed(&origin);
    }

    fn start() {
        // The check for IS_INTERACTIVE_SESSION here is not redundant with the one in
        // raw_win_event_hook_recv. On a session switch, Windows fires events for ALL
        // session windows simultaneously, flooding the unbounded channel before
        // WM_WTSSESSION_CHANGE can be delivered to BgWindowProc (which runs on a
        // separate thread). The dispatcher thread then drains that backlog calling
        // expensive Win32 APIs (as_focused_app_information, etc.) for every queued
        // event even though IS_INTERACTIVE_SESSION is already false. This check makes
        // each backlogged event a cheap no-op (atomic load + branch) instead.
        let eid = Self::subscribe(|(event, mut origin)| {
            if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
                return;
            }
            if event == WinEvent::SystemForeground {
                origin = Window::get_foregrounded(); // sometimes this event is emited with the wrong origin
            }
            Self::legacy_process_event(event, origin);
        });
        Self::set_event_handler_priority(&eid, 1);

        spawn_named_thread("WinEventHook", move || unsafe {
            SetWinEventHook(
                EVENT_MIN,
                EVENT_MAX,
                None,
                Some(HookManager::raw_win_event_hook_recv),
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
    fn log_event(event: WinEvent, origin: Window) {
        use owo_colors::OwoColorize;
        if event == WinEvent::ObjectLocationChange {
            return;
        }
        let event_value = event.green();
        if event == WinEvent::ObjectDestroy {
            return println!("{event_value:?}({:0x})", origin.address());
        }
        println!("{event_value:?} | {origin:?}");
    }

    fn legacy_process_event(event: WinEvent, origin: Window) {
        // Self::log_event(event, origin);

        // TODO: optimize this, update independent fields instead of the whole struct
        {
            let shoup_update_focused = matches!(
                event,
                WinEvent::SystemForeground
                    | WinEvent::SynThrottledForegroundRectChange
                    | WinEvent::ObjectNameChange
                    | WinEvent::SystemMoveSizeStart
                    | WinEvent::SystemMoveSizeEnd
            );

            if shoup_update_focused && origin.is_focused() {
                emit_to_webviews(
                    SeelenEvent::GlobalFocusChanged,
                    origin.as_focused_app_information(),
                );
            }
        }
    }
}

pub fn register_win_hook() -> Result<()> {
    log::trace!("Registering Windows and Virtual Desktop Hooks");
    init_zombie_window_killer();
    HookManager::start();

    let eid = HookManager::subscribe(|(event, origin)| match event {
        WinEvent::SystemMoveSizeStart => {
            origin.set_dragging(true);
        }
        WinEvent::SystemMoveSizeEnd => {
            origin.set_dragging(false);
        }
        _ => (),
    });
    HookManager::set_event_handler_priority(&eid, 3);

    // todo move this to input/mouse/keyboard module
    spawn_named_thread("MouseEventHook", || {
        let mut last_pos = seelen_core::Point::default();
        let sleep_time = Duration::from_millis(100); // 10fps
        loop {
            // Pause when session is not interactive to reduce CPU usage
            if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }

            if let Ok(pos) = Mouse::get_cursor_pos() {
                if last_pos != pos {
                    emit_to_webviews(SeelenEvent::GlobalMouseMove, &[pos.x, pos.y]);
                    last_pos = pos;
                }
            }
            std::thread::sleep(sleep_time);
        }
    });

    Ok(())
}

pub fn init_zombie_window_killer() {
    let existing_windows = Arc::new(RwLock::new(HashSet::new()));

    let dict = existing_windows.clone();
    HookManager::subscribe(move |(event, origin)| match event {
        WinEvent::ObjectCreate => {
            dict.write().insert(origin.address());
        }
        WinEvent::ObjectDestroy => {
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
            .for_each_and_descendants(|window| {
                existing_windows.write().insert(window.address());
            })
            .log_error();

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));

            // Pause when session is not interactive to reduce CPU usage
            if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
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
                HookManager::send((WinEvent::ObjectDestroy, window));
            }
        }
    });
}
