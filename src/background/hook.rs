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
use tauri::Emitter;
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
    app::{get_app_handle, Seelen, SEELEN},
    error::{ErrorMap, Result, ResultLogExt},
    event_manager, log_error,
    modules::input::Mouse,
    state::application::FULL_STATE,
    trace_lock,
    utils::spawn_named_thread,
    widgets::{weg::SeelenWeg, window_manager::instance::WindowManagerV2},
    windows_api::{
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
        if id_object != OBJID_WINDOW.0 || !Seelen::is_running() {
            return;
        }

        // deprecated code refactor this on future.
        if FULL_STATE.load().is_weg_enabled() {
            // raw events should be only used for a fastest and immediately processing
            log_error!(SeelenWeg::process_raw_win_event(event, origin));
        }

        let event = WinEvent::from(event);
        let origin = Window::from(origin);
        Self::send((event, origin));

        for event in event.get_synthetics(&origin).unwrap_or_default() {
            Self::send((event, origin));
        }
    }

    fn start() {
        let eid = Self::subscribe(|(event, mut origin)| {
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

    fn log_event(event: WinEvent, origin: Window) {
        if event == WinEvent::ObjectLocationChange || !LOG_WIN_EVENTS.load(Ordering::Acquire) {
            return;
        }
        let event_value = {
            #[cfg(dev)]
            {
                use owo_colors::OwoColorize;
                event.green()
            }
            #[cfg(not(dev))]
            {
                &event
            }
        };
        if event == WinEvent::ObjectDestroy {
            return log::debug!("{event_value:?}({:0x})", origin.address());
        }
        log::debug!("{event_value:?} | {origin:?}");
    }

    fn legacy_process_event(event: WinEvent, origin: Window) {
        Self::log_event(event, origin);

        // TODO: optimize this, update independent fields instead of the whole struct
        {
            let shoup_update_focused = matches!(
                event,
                WinEvent::SystemForeground
                    | WinEvent::SyntheticForegroundLocationChange
                    | WinEvent::ObjectNameChange
                    | WinEvent::SystemMoveSizeStart
                    | WinEvent::SystemMoveSizeEnd
                    | WinEvent::SyntheticMaximizeStart
                    | WinEvent::SyntheticMaximizeEnd
                    | WinEvent::SyntheticFullscreenStart
                    | WinEvent::SyntheticFullscreenEnd
            );

            if shoup_update_focused && origin.is_focused() {
                get_app_handle()
                    .emit(
                        SeelenEvent::GlobalFocusChanged,
                        origin.as_focused_app_information(),
                    )
                    .wrap_error()
                    .log_error();
            }
        }

        let app_state = FULL_STATE.load();
        if app_state.is_weg_enabled() {
            log_error!(SeelenWeg::process_global_win_event(event, &origin), event);
        }

        if app_state.is_window_manager_enabled() {
            log_error!(WindowManagerV2::process_win_event(event, &origin), event);
        }

        {
            let mut seelen = trace_lock!(SEELEN);
            if let Some(wall) = &mut seelen.wall {
                log_error!(wall.process_win_event(event, &origin), event);
            }
        };

        let mut seelen = trace_lock!(SEELEN);
        for instance in seelen.instances_mut() {
            if let Some(toolbar) = &mut instance.toolbar {
                log_error!(toolbar.process_win_event(event, &origin), event);
            }
            if let Some(weg) = &mut instance.weg {
                log_error!(weg.process_individual_win_event(event, &origin), event);
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
        let handle = get_app_handle();
        let mut last_pos = seelen_core::Point::default();
        let sleep_time = Duration::from_millis(100); // 10fps
        loop {
            if let Ok(pos) = Mouse::get_cursor_pos() {
                if last_pos != pos {
                    let _ = handle.emit(SeelenEvent::GlobalMouseMove, &[pos.x, pos.y]);
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
            let guard = existing_windows.write();
            for addr in guard.iter() {
                let window = Window::from(*addr);
                if !window.is_window() {
                    log::trace!("Reaping window: {:0x}", window.address());
                    HookManager::send((WinEvent::ObjectDestroy, window));
                }
            }
        }
    });
}
