use std::{thread::sleep, time::Duration, sync::Arc};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, EVENT_MAX, EVENT_MIN,
            EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_FOCUS,
            EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE,
            EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND,
            EVENT_SYSTEM_MOVESIZESTART, MSG,
        },
    },
};
use winvd::{listen_desktop_events, DesktopEvent};

use crate::{
    apps_config::{AppExtraFlag, SETTINGS_BY_APP}, error_handler::{log_if_error, Result}, seelen::SEELEN, seelen_bar::FancyToolbar, seelen_weg::SeelenWeg, seelen_wm::WindowManager, utils::{constants::{FORCE_RETILING_AFTER_ADD, IGNORE_FOCUS}, sleep_millis}, windows_api::WindowsApi
};

lazy_static! {
    pub static ref HOOK_MANAGER: Arc<Mutex<HookManager>> = Arc::new(Mutex::new(HookManager::new()));
}

pub struct HookManager {
    paused: bool,
    waiting_event: Option<u32>,
    waiting_hwnd: Option<HWND>,
    resume_cb: Option<Box<dyn Fn(&mut HookManager) + Send>>,
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

    pub fn pause_and_resume_after(&mut self, event: u32, hwnd: HWND) {
        self.pause();
        self.waiting_event = Some(event);
        self.waiting_hwnd = Some(hwnd);
    }

    pub fn set_resume_callback<F>(&mut self, cb: F)
    where 
        F: Fn(&mut HookManager) + Send + 'static
    {
        self.resume_cb = Some(Box::new(cb));
    }

    pub fn is_waiting_for(&self, event: u32, hwnd: HWND) -> bool {
        self.waiting_event == Some(event) && self.waiting_hwnd == Some(hwnd)
    }

    pub fn emit_fake_win_event(&mut self, event: u32, hwnd: HWND) {
        std::thread::spawn(move || {
            win_event_hook(HWINEVENTHOOK::default(), event, hwnd, 0, 0, 0, 0);
        });
    }
}



pub fn process_vd_event(event: DesktopEvent) -> Result<()> {
    match event {
        DesktopEvent::DesktopChanged{ new, old: _ } => {
            let mut seelen = SEELEN.lock();
            if let Some(wm) = seelen.wm_mut() {
                wm.discard_reservation()?;
                wm.set_active_workspace(format!("{:?}", new.get_id()?))?;
            }
        }
        DesktopEvent::WindowChanged(hwnd) => {
            if WindowsApi::is_window(hwnd) {
                if let Some(config) = SETTINGS_BY_APP.lock().get_by_window(hwnd) {
                    if config.options_contains(AppExtraFlag::Pinned) && !winvd::is_pinned_window(hwnd)? {
                        winvd::pin_window(hwnd)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn process_win_event(event: u32, hwnd: HWND) -> Result<()> {
    match event {
        EVENT_SYSTEM_MOVESIZESTART => {
            let seelen = SEELEN.lock();
            if let Some(wm) = seelen.wm() {
                if wm.is_managed(hwnd) {
                    wm.pseudo_pause()?;
                }
            }
        }
        EVENT_SYSTEM_MOVESIZEEND => {
            let seelen = SEELEN.lock();
            if let Some(wm) = seelen.wm() {
                if wm.is_managed(hwnd) {
                    wm.force_retiling()?;
                    sleep(Duration::from_millis(35));
                    wm.pseudo_resume()?;
                }
            }
        }
        EVENT_SYSTEM_MINIMIZEEND => {
            let mut seelen = SEELEN.lock();
            if let Some(wm) = seelen.wm_mut() {
                if !wm.is_managed(hwnd) && WindowManager::should_manage(hwnd) {
                    wm.add_hwnd(hwnd)?;
                }
            }
        }
        EVENT_SYSTEM_MINIMIZESTART => {
            let mut seelen = SEELEN.lock();
            if let Some(wm) = seelen.wm_mut() {
                if wm.is_managed(hwnd) {
                    wm.remove_hwnd(hwnd)?;
                }
            }
        }
        EVENT_OBJECT_HIDE => {
            let mut seelen = SEELEN.lock();
            if let Some(weg) = seelen.weg_mut() {
                if weg.contains_app(hwnd) {
                    // We filter apps with parents but UWP apps using ApplicationFrameHost.exe are initialized without
                    // parent so we can't filter it on open event but these are immediately hidden when the ApplicationFrameHost.exe parent
                    // is assigned to the window. After that we replace the window hwnd to its parent and remove child from the list
                    let parent = WindowsApi::get_parent(hwnd);
                    if parent.0 != 0 {
                        weg.replace_hwnd(hwnd, parent)?;
                    } else {
                        weg.remove_hwnd(hwnd);
                    }
                }
            }

            if let Some(wm) = seelen.wm_mut() {
                if wm.is_managed(hwnd) {
                    wm.remove_hwnd(hwnd)?;
                }
            }
        }
        EVENT_OBJECT_DESTROY /* | EVENT_OBJECT_CLOAKED */ => {
            let mut seelen = SEELEN.lock();

            if let Some(weg) = seelen.weg_mut() {
                if weg.contains_app(hwnd) {
                    weg.remove_hwnd(hwnd);
                }
            }

            if let Some(wm) = seelen.wm_mut() {
                let title = WindowsApi::get_window_text(hwnd);
                if WindowManager::VIRTUAL_PREVIEWS.contains(&title.as_str()) {
                    wm.pseudo_resume()?;
                }
                if wm.is_managed(hwnd) {
                    wm.remove_hwnd(hwnd)?;
                }
            }
        }
        EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE /* | EVENT_OBJECT_UNCLOAKED */ => {
            let mut seelen = SEELEN.lock();
            if let Some(weg) = seelen.weg_mut() {
                if "Shell_TrayWnd" == WindowsApi::get_class(hwnd)? {
                    // ensure that the taskbar is always hidden
                    weg.hide_taskbar(true);
                }

                if SeelenWeg::is_real_window(hwnd, false) {
                    weg.add_hwnd(hwnd);
                }
            }

            if let Some(wm) = seelen.wm_mut() {
                let title = WindowsApi::get_window_text(hwnd);
                if WindowManager::VIRTUAL_PREVIEWS.contains(&title.as_str()) {
                    wm.pseudo_pause()?;
                }

                if !wm.is_managed(hwnd) && WindowManager::should_manage(hwnd) {
                    wm.set_active_window(hwnd)?;
                    if wm.add_hwnd(hwnd)? && FORCE_RETILING_AFTER_ADD.contains(&title) {
                        // Todo search a better way to do this
                        std::thread::spawn(|| -> Result<()> {
                            sleep_millis(250);
                            SEELEN.lock().wm().unwrap().force_retiling()?;
                            Ok(())
                        });
                    };
                }
            }
        }
        EVENT_OBJECT_NAMECHANGE => {
            let mut seelen = SEELEN.lock();
            if let Some(weg) = seelen.weg_mut() {
                if weg.contains_app(hwnd) {
                    weg.update_app(hwnd);
                } else if SeelenWeg::is_real_window(hwnd, false) {
                    weg.add_hwnd(hwnd);
                }
            }
            
            if let Some(wm) = seelen.wm_mut() {
                if !wm.is_managed(hwnd) && WindowManager::should_manage(hwnd) {
                    wm.set_active_window(hwnd)?;
                    let title = WindowsApi::get_window_text(hwnd);
                    if wm.add_hwnd(hwnd)? && FORCE_RETILING_AFTER_ADD.contains(&title) {
                        // Todo search a better way to do this
                        std::thread::spawn(|| -> Result<()> {
                            sleep_millis(250);
                            SEELEN.lock().wm().unwrap().force_retiling()?;
                            Ok(())
                        });
                    };
                }
            }
        }
        EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND => {
            let mut seelen = SEELEN.lock();
            if let Some(seelenweg) = seelen.weg_mut() {
                match seelenweg.contains_app(hwnd) {
                    true => seelenweg.set_active_window(hwnd)?,
                    false => seelenweg.set_active_window(HWND(0))?, // avoid rerenders on multiple unmanaged focus
                }
                seelenweg.update_status_if_needed(hwnd)?;
            }
            if let Some(wm) = seelen.wm_mut() {
                wm.set_active_window(hwnd)?;
            }
        }
        EVENT_OBJECT_LOCATIONCHANGE => {
            let mut seelen = SEELEN.lock();
            if let Some(weg) = seelen.weg_mut() {
                weg.update_status_if_needed(hwnd)?;
            }

            if let Some(wm) = seelen.wm_mut() {
                if WindowsApi::is_maximized(hwnd) {
                    wm.pseudo_pause()?;
                }
            }
        }
        _ => {}
    };

    Ok(())
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

    let mut hook_manager = HOOK_MANAGER.lock();
    if hook_manager.is_paused() {
        if hook_manager.is_waiting_for(event, hwnd) {
            hook_manager.resume();
        }
        return;
    }

    /* if event == EVENT_OBJECT_LOCATIONCHANGE {
        return;
    }

    let winevent = match crate::winevent::WinEvent::try_from(event) {
        Ok(event) => event,
        Err(_) => return,
    };

    println!(
        "{:?} || {} || {} || {}",
        winevent,
        WindowsApi::exe(hwnd).unwrap_or_default(),
        WindowsApi::get_class(hwnd).unwrap_or_default(),
        WindowsApi::get_window_text(hwnd)
    ); */

    let title = WindowsApi::get_window_text(hwnd);
    if (event == EVENT_OBJECT_FOCUS || event == EVENT_SYSTEM_FOREGROUND) && IGNORE_FOCUS.contains(&title) {
        return;
    }

    log_if_error(FancyToolbar::process_win_event(event, hwnd));
    log_if_error(process_win_event(event, hwnd));
}

pub fn register_win_hook() -> Result<()> {
    std::thread::spawn(move || unsafe {
        SetWinEventHook(EVENT_MIN, EVENT_MAX, None, Some(win_event_hook), 0, 0, 0);

        let mut msg: MSG = MSG::default();
        loop {
            if !GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                log::info!("windows event processing shutdown");
                break;
            };
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
            std::thread::sleep(Duration::from_millis(10));
        }
    });

    std::thread::spawn(move || -> Result<()> {
        let (sender, receiver) = std::sync::mpsc::channel::<DesktopEvent>();
        let _notifications_thread = listen_desktop_events(sender)?;
        for event in receiver {
            log_if_error(process_vd_event(event))
        }
        Ok(())
    });
    Ok(())
}
