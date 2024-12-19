use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicIsize, Ordering},
        Arc,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use itertools::Itertools;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::handlers::SeelenEvent;
use serde::Serialize;
use tauri::Emitter;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, EVENT_MAX, EVENT_MIN, MSG,
        },
    },
};

use crate::{
    error_handler::Result,
    log_error,
    modules::{
        input::{domain::Point, Mouse},
        virtual_desk::{get_vd_manager, VirtualDesktopEvent, VirtualDesktopManager},
    },
    seelen::{get_app_handle, Seelen, SEELEN},
    seelen_weg::SeelenWeg,
    seelen_wm_v2::instance::WindowManagerV2,
    state::{application::FULL_STATE, domain::AppExtraFlag},
    trace_lock,
    utils::spawn_named_thread,
    windows_api::{window::Window, WindowsApi},
    winevent::WinEvent,
};

lazy_static! {
    static ref HOOK_MANAGER_SKIPPER: Arc<Mutex<HookManagerSkipper>> = Arc::new(Mutex::new(HookManagerSkipper::default()));
    // Last active window omitting all the seelen overlays
    pub static ref LAST_ACTIVE_NOT_SEELEN: AtomicIsize = AtomicIsize::new(WindowsApi::get_foreground_window().0 as _);
}

pub static LOG_WIN_EVENTS: AtomicBool = AtomicBool::new(false);

#[derive(Serialize, Clone)]
pub struct FocusedApp {
    hwnd: isize,
    title: String,
    name: String,
    exe: Option<PathBuf>,
}

#[derive(Debug)]
pub struct HookManagerSkipperItem {
    hwnd: isize,
    event: WinEvent,
    expiry: Instant,
    skipped: bool,
}

#[derive(Debug, Default)]
pub struct HookManagerSkipper {
    to_skip: Vec<HookManagerSkipperItem>,
}

impl HookManagerSkipper {
    /// this function is intended to be called before ejecuting a actions that will
    /// trigger some window event, so the skipper will be removed after 1 second
    /// if no event is emitted in that interval
    pub fn skip(&mut self, event: WinEvent, hwnd: HWND) {
        self.to_skip.push(HookManagerSkipperItem {
            hwnd: hwnd.0 as _,
            event,
            expiry: Instant::now() + Duration::from_millis(1000),
            skipped: false,
        });
    }

    fn cleanup(&mut self) {
        self.to_skip
            .retain(|s| !s.skipped && s.expiry > Instant::now());
    }

    fn should_skip(&mut self, event: WinEvent, hwnd: HWND) -> bool {
        // skip foreground on invisible windows
        if event == WinEvent::SystemForeground && !WindowsApi::is_window_visible(hwnd) {
            return true;
        }

        let hwnd = hwnd.0 as isize;
        if let Some(skipper) = self
            .to_skip
            .iter_mut()
            .find(|s| s.hwnd == hwnd && s.event == event && !s.skipped && s.expiry > Instant::now())
        {
            skipper.skipped = true;
            return true;
        }
        false
    }
}

pub struct HookManager;
impl HookManager {
    pub fn run_with_async<F, T>(f: F) -> JoinHandle<T>
    where
        F: FnOnce(&mut HookManagerSkipper) -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        std::thread::spawn(move || f(&mut *trace_lock!(HOOK_MANAGER_SKIPPER)))
    }

    fn log_skipped(event: WinEvent) {
        if LOG_WIN_EVENTS.load(Ordering::Relaxed) {
            log::debug!("Skipping WinEvent::{:?}", event);
        }
    }

    fn log_event(event: WinEvent, origin: HWND) {
        if !LOG_WIN_EVENTS.load(Ordering::Relaxed) || event == WinEvent::ObjectLocationChange {
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

        log::debug!(
            "{:?}({:?}) || {} || {} || {}",
            event_value,
            origin.0,
            WindowsApi::exe(origin).unwrap_or_default(),
            WindowsApi::get_class(origin).unwrap_or_default(),
            WindowsApi::get_window_text(origin),
        );
    }

    fn _event(event: WinEvent, origin: HWND) {
        Self::log_event(event, origin);

        {
            let mut hook_manager = trace_lock!(HOOK_MANAGER_SKIPPER);
            if hook_manager.should_skip(event, origin) {
                Self::log_skipped(event);
                hook_manager.cleanup();
                return;
            }
        }

        let window = Window::from(origin);
        if event == WinEvent::SystemForeground && !window.is_seelen_overlay() {
            LAST_ACTIVE_NOT_SEELEN.store(origin.0 as _, Ordering::Relaxed);
        }

        if event == WinEvent::ObjectFocus || event == WinEvent::SystemForeground {
            let title = window.title();
            log_error!(get_app_handle().emit(
                SeelenEvent::GlobalFocusChanged,
                FocusedApp {
                    title,
                    hwnd: origin.0 as _,
                    name: window
                        .app_display_name()
                        .unwrap_or(String::from("Error on App Name")),
                    exe: window.exe().ok(),
                },
            ));
        }

        let log_error_event = move |name: &str, result: Result<()>| {
            if let Err(err) = result {
                log::error!(
                    "{} => Event: {:?} Error: {:?} Window: {:?}",
                    name,
                    event,
                    err,
                    window
                );
            }
        };

        if let VirtualDesktopManager::Seelen(vd) = get_vd_manager().as_ref() {
            log_error_event("Virtual Desk", vd.on_win_event(event, &window));
        }

        let app_state = FULL_STATE.load();
        if app_state.is_weg_enabled() {
            std::thread::spawn(move || {
                log_error_event(
                    "Weg Global",
                    SeelenWeg::process_global_win_event(event, &window),
                );
            });
        }

        if app_state.is_window_manager_enabled() {
            std::thread::spawn(move || {
                log_error_event(
                    "WM Global",
                    WindowManagerV2::process_win_event(event, &window),
                );
            });
        }

        {
            let mut seelen = trace_lock!(SEELEN);
            if let Some(wall) = seelen.wall_mut() {
                log_error_event("Wall Instance", wall.process_win_event(event, &window));
            }
        };

        let mut seelen = trace_lock!(SEELEN);
        for instance in seelen.instances_mut() {
            if let Some(toolbar) = instance.toolbar_mut() {
                log_error_event("Toolbar Instance", toolbar.process_win_event(event, origin));
            }

            if let Some(weg) = instance.weg_mut() {
                log_error_event(
                    "Weg Instance",
                    weg.process_individual_win_event(event, origin),
                );
            }
        }
    }

    pub fn emit_event(event: WinEvent, origin: HWND) {
        HookManager::_event(event, origin);
        if let Ok(synthetics) = event.get_synthetics(origin) {
            for synthetic_event in synthetics {
                HookManager::_event(synthetic_event, origin)
            }
        }
    }
}

pub fn process_vd_event(event: VirtualDesktopEvent) -> Result<()> {
    if FULL_STATE.load().is_window_manager_enabled() {
        log_error!(WindowManagerV2::process_vd_event(&event));
    }

    match event {
        VirtualDesktopEvent::DesktopCreated(_)
        | VirtualDesktopEvent::DesktopDestroyed {
            destroyed: _,
            fallback: _,
        }
        | VirtualDesktopEvent::DesktopMoved {
            desktop: _,
            old_index: _,
            new_index: _,
        }
        | VirtualDesktopEvent::DesktopNameChanged(_, _) => {
            let desktops = get_vd_manager()
                .get_all()?
                .iter()
                .map(|d| d.as_serializable())
                .collect_vec();
            get_app_handle().emit(SeelenEvent::WorkspacesChanged, &desktops)?;
        }

        VirtualDesktopEvent::DesktopChanged { new, old: _ } => {
            get_app_handle().emit(SeelenEvent::ActiveWorkspaceChanged, new.id())?;
        }
        VirtualDesktopEvent::WindowChanged(window) => {
            let hwnd = HWND(window as _);
            if WindowsApi::is_window(hwnd) {
                if let Some(config) = FULL_STATE.load().get_app_config_by_window(hwnd) {
                    let vd = get_vd_manager();
                    if config.options.contains(&AppExtraFlag::Pinned)
                        && !vd.is_pinned_window(window)?
                    {
                        vd.pin_window(window)?;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

lazy_static! {
    static ref DICT: Arc<Mutex<HashMap<isize, Instant>>> = Arc::new(Mutex::new(HashMap::new()));
}
static LAST_LOCATION_CHANGED: AtomicIsize = AtomicIsize::new(0);

pub fn location_delay_completed(origin: HWND) -> bool {
    let last = LAST_LOCATION_CHANGED.load(Ordering::Acquire);
    let mut dict = trace_lock!(DICT);

    let should_continue = match dict.entry(origin.0 as _) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
            if last != origin.0 as isize || entry.get().elapsed() > Duration::from_millis(50) {
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
        LAST_LOCATION_CHANGED.store(origin.0 as _, Ordering::Release);
    }

    should_continue
}

pub extern "system" fn win_event_hook(
    hook_handle: HWINEVENTHOOK,
    event: u32,
    origin: HWND,
    id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    let hook_was_invalidated = hook_handle.is_invalid();
    if !Seelen::is_running() {
        if !hook_was_invalidated {
            log::trace!("Exiting WinEventHook");
            let _ = unsafe { UnhookWinEvent(hook_handle) };
        }
        return;
    }

    if id_object != 0 {
        return;
    }

    if FULL_STATE.load().is_weg_enabled() {
        // raw events should be only used for a fastest and immediately processing
        log_error!(SeelenWeg::process_raw_win_event(event, origin));
    }

    let event = WinEvent::from(event);
    if event == WinEvent::ObjectLocationChange && !location_delay_completed(origin) {
        return;
    }
    HookManager::emit_event(event, origin)
}

pub fn register_win_hook() -> Result<()> {
    log::trace!("Registering Windows and Virtual Desktop Hooks");

    spawn_named_thread("WinEventHook", move || unsafe {
        SetWinEventHook(EVENT_MIN, EVENT_MAX, None, Some(win_event_hook), 0, 0, 0);
        let mut msg: MSG = MSG::default();
        loop {
            if !GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                break;
            };
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    })?;

    let (sender, receiver) = std::sync::mpsc::channel::<VirtualDesktopEvent>();
    get_vd_manager().listen_events(sender)?;
    spawn_named_thread("VirtualDesktopEventHook", move || {
        for event in receiver {
            log_error!(process_vd_event(event))
        }
    })?;

    spawn_named_thread("MouseEventHook", || {
        let handle = get_app_handle();
        let mut last_pos = Point::default();
        loop {
            if let Ok(pos) = Mouse::get_cursor_pos() {
                if last_pos != pos {
                    let _ = handle.emit(SeelenEvent::GlobalMouseMove, &[pos.get_x(), pos.get_y()]);
                    last_pos = pos;
                }
            }
            std::thread::sleep(Duration::from_millis(66)); // 15 FPS
        }
    })?;

    Ok(())
}
