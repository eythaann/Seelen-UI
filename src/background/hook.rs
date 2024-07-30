use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicIsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use color_eyre::owo_colors::OwoColorize;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::Emitter;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, EVENT_MAX, EVENT_MIN, MSG,
        },
    },
};
use winvd::{listen_desktop_events, DesktopEvent};

use crate::{
    apps_config::{AppExtraFlag, SETTINGS_BY_APP},
    error_handler::Result,
    log_error,
    seelen::{Seelen, SEELEN},
    seelen_weg::{SeelenWeg, TASKBAR_CLASS},
    trace_lock,
    utils::{constants::IGNORE_FOCUS, is_windows_11},
    windows_api::WindowsApi,
    winevent::WinEvent,
};

lazy_static! {
    pub static ref HOOK_MANAGER: Arc<Mutex<HookManager>> = Arc::new(Mutex::new(HookManager::new()));
}

type HookCallback = Box<dyn Fn(&mut HookManager) + Send + 'static>;
pub struct HookManager {
    paused: bool,
    waiting_event: Option<WinEvent>,
    waiting_hwnd: Option<HWND>,
    resume_cb: Option<HookCallback>,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            paused: false,
            waiting_event: None,
            waiting_hwnd: None,
            resume_cb: None,
        }
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
        if let Some(cb) = self.resume_cb.take() {
            cb(self);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause_and_resume_after(&mut self, event: WinEvent, hwnd: HWND) {
        self.pause();
        self.waiting_event = Some(event);
        self.waiting_hwnd = Some(hwnd);
    }

    pub fn set_resume_callback<F>(&mut self, cb: F)
    where
        F: Fn(&mut HookManager) + Send + 'static,
    {
        self.resume_cb = Some(Box::new(cb));
    }

    pub fn is_waiting_for(&self, event: WinEvent, hwnd: HWND) -> bool {
        self.waiting_event == Some(event) && self.waiting_hwnd == Some(hwnd)
    }

    pub fn emit_fake_win_event(&mut self, event: u32, hwnd: HWND) {
        std::thread::spawn(move || {
            win_event_hook(HWINEVENTHOOK::default(), event, hwnd, 0, 0, 0, 0);
        });
    }

    pub fn _log_event(event: WinEvent, origin: HWND) {
        if event == WinEvent::ObjectLocationChange {
            return;
        }

        println!(
            "{:?}({:x}) || {} || {} || {:<20}",
            event.green(),
            origin.0,
            WindowsApi::exe(origin).unwrap_or_default(),
            WindowsApi::get_class(origin).unwrap_or_default(),
            WindowsApi::get_window_text(origin),
        );
    }

    pub fn event(&mut self, event: WinEvent, origin: HWND) {
        // uncomment for debug
        // Self::_log_event(event, origin);

        if self.is_paused() {
            if self.is_waiting_for(event, origin) {
                self.resume();
            }
            return;
        }

        let title = WindowsApi::get_window_text(origin);
        if (event == WinEvent::ObjectFocus || event == WinEvent::SystemForeground)
            && IGNORE_FOCUS.contains(&title)
        {
            return;
        }

        let mut seelen = trace_lock!(SEELEN);
        log_error!(seelen.process_win_event(event, origin));

        for monitor in seelen.monitors_mut() {
            if let Some(toolbar) = monitor.toolbar_mut() {
                log_error!(toolbar.process_win_event(event, origin));
            }

            if let Some(weg) = monitor.weg_mut() {
                log_error!(weg.process_win_event(event, origin));
            }

            if let Some(wm) = monitor.wm_mut() {
                log_error!(wm.process_win_event(event, origin));
            }
        }
    }
}

pub fn process_vd_event(event: DesktopEvent) -> Result<()> {
    let mut seelen = trace_lock!(SEELEN);
    for monitor in seelen.monitors_mut() {
        if let Some(wm) = monitor.wm_mut() {
            log_error!(wm.process_vd_event(&event));
        }
    }

    match event {
        DesktopEvent::DesktopCreated(_)
        | DesktopEvent::DesktopDestroyed {
            destroyed: _,
            fallback: _,
        }
        | DesktopEvent::DesktopMoved {
            desktop: _,
            old_index: _,
            new_index: _,
        }
        | DesktopEvent::DesktopNameChanged(_, _) => {
            let desktops = winvd::get_desktops()?;
            let mut desktops_names = Vec::new();
            for (i, d) in desktops.iter().enumerate() {
                if let Ok(name) = d.get_name() {
                    desktops_names.push(name);
                } else {
                    desktops_names.push(format!("Desktop {}", i + 1))
                }
            }
            seelen.handle().emit("workspaces-changed", desktops_names)?;
        }

        DesktopEvent::DesktopChanged { new, old: _ } => {
            seelen
                .handle()
                .emit("active-workspace-changed", new.get_index()?)?;
        }
        _ => {}
    }

    if let DesktopEvent::WindowChanged(hwnd) = event {
        if WindowsApi::is_window(hwnd) {
            if let Some(config) = trace_lock!(SETTINGS_BY_APP).get_by_window(hwnd) {
                if config.options_contains(AppExtraFlag::Pinned) && !winvd::is_pinned_window(hwnd)?
                {
                    winvd::pin_window(hwnd)?;
                }
            }
        }
    }

    Ok(())
}

impl Seelen {
    pub fn process_win_event(&mut self, event: WinEvent, origin: HWND) -> Result<()> {
        match event {
            WinEvent::ObjectShow | WinEvent::ObjectCreate => {
                // ensure that the taskbar is always hidden
                if self.state().is_weg_enabled() {
                    let class = WindowsApi::get_class(origin)?;
                    let parent_class =
                        WindowsApi::get_class(WindowsApi::get_parent(origin)).unwrap_or_default();
                    if TASKBAR_CLASS.contains(&class.as_str())
                        || TASKBAR_CLASS.contains(&parent_class.as_str())
                    {
                        SeelenWeg::hide_taskbar(true);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

lazy_static! {
    static ref DICT: Arc<Mutex<HashMap<isize, Instant>>> = Arc::new(Mutex::new(HashMap::new()));
}
static LAST_LOCATION_CHANGED: AtomicIsize = AtomicIsize::new(0);

pub fn location_delay_completed(origin: HWND) -> bool {
    let last = LAST_LOCATION_CHANGED.load(Ordering::Acquire);
    let mut dict = DICT.lock();

    let should_continue = match dict.entry(origin.0) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            if last != origin.0 || entry.get().elapsed() > Duration::from_millis(50) {
                entry.insert(Instant::now());
                true
            } else {
                false
            }
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
            entry.insert(Instant::now());
            true
        }
    };

    if should_continue {
        LAST_LOCATION_CHANGED.store(origin.0, Ordering::Release);
    }

    should_continue
}

pub extern "system" fn win_event_hook(
    _h_win_event_hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if id_object != 0 {
        return;
    }

    let event = match WinEvent::try_from(event) {
        Ok(event) => event,
        Err(_) => return,
    };

    if event == WinEvent::ObjectLocationChange && !location_delay_completed(hwnd) {
        return;
    }

    let mut hook_manager = HOOK_MANAGER.lock();
    hook_manager.event(event, hwnd);

    if let Some(synthetic_event) = event.get_synthetic(hwnd) {
        hook_manager.event(synthetic_event, hwnd);
    }
}

pub fn register_win_hook() -> Result<()> {
    log::trace!("Registering Windows and Virtual Desktop Hooks");

    let stack_size = 5 * 1024 * 1024; // 5 MB
    std::thread::Builder::new()
        .name("win_event_hook".into())
        .stack_size(stack_size)
        .spawn(move || unsafe {
            SetWinEventHook(EVENT_MIN, EVENT_MAX, None, Some(win_event_hook), 0, 0, 0);

            let mut msg: MSG = MSG::default();
            loop {
                if !GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                    log::info!("windows event processing shutdown");
                    break;
                };
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
                std::thread::sleep(Duration::from_millis(10));
            }
        })?;

    // Todo search why virtual desktop events are not working on windows 10
    if is_windows_11() {
        std::thread::spawn(move || -> Result<()> {
            let (sender, receiver) = std::sync::mpsc::channel::<DesktopEvent>();
            let _notifications_thread = listen_desktop_events(sender)?;
            for event in receiver {
                log_error!(process_vd_event(event))
            }
            Ok(())
        });
    }

    Ok(())
}
