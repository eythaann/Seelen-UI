use std::time::Duration;

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, EVENT_MAX, EVENT_MIN,
            EVENT_OBJECT_CLOAKED, EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_FOCUS,
            EVENT_OBJECT_HIDE, EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
            EVENT_SYSTEM_FOREGROUND, MSG,
        },
    },
};

use crate::{error_handler::Result, seelen::SEELEN, seelenweg::SeelenWeg, windows_api::WindowsApi};

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

    /*
    if event == EVENT_OBJECT_LOCATIONCHANGE {
        return;
    }

    let winevent = match WinEvent::try_from(event) {
        Ok(event) => event,
        Err(_) => return,
    };

    println!("{:?}", winevent); */

    match event {
        EVENT_OBJECT_HIDE => {
            let mut seelen = SEELEN.lock();
            if seelen.weg().contains_app(hwnd) {
                // We filter apps with parents but UWP apps using ApplicationFrameHost.exe are initialized without
                // parent so we can't filter it on open event but these are inmediatly hidden when the ApplicationFrameHost.exe parent
                // is assigned to the window. After that we replace the window hwnd to its parent and remove child from the list
                let parent = WindowsApi::get_parent(hwnd);
                if parent.0 != 0 {
                    seelen.weg_mut().replace_hwnd(hwnd, parent);
                } else {
                    seelen.weg_mut().remove_hwnd(hwnd);
                }
            }
        }
        EVENT_OBJECT_DESTROY | EVENT_OBJECT_CLOAKED => {
            let mut seelen = SEELEN.lock();
            if seelen.weg().contains_app(hwnd) {
                seelen.weg_mut().remove_hwnd(hwnd);
            }
        }
        EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE | EVENT_OBJECT_UNCLOAKED => {
            if SeelenWeg::should_handle_hwnd(hwnd) {
                let mut seelen = SEELEN.lock();
                seelen.weg_mut().add_hwnd(hwnd);
            }
        }
        EVENT_OBJECT_NAMECHANGE => {
            let mut seelen = SEELEN.lock();
            if seelen.weg().contains_app(hwnd) {
                seelen.weg_mut().update_app(hwnd);
            } else if SeelenWeg::should_handle_hwnd(hwnd) {
                seelen.weg_mut().add_hwnd(hwnd);
            }
        }
        EVENT_OBJECT_FOCUS | EVENT_SYSTEM_FOREGROUND => {
            let seelen = SEELEN.lock();
            let seelenweg = seelen.weg();
            if seelenweg.contains_app(hwnd) {
                seelenweg.set_focused(hwnd);
            } else if WindowsApi::get_window_text(hwnd) != "Task Switching" {
                seelenweg.set_focused(HWND(0));
            }
        }
        _ => {}
    }
}

pub fn register_hook() -> Result<()> {
    std::thread::spawn(move || {
        unsafe { SetWinEventHook(EVENT_MIN, EVENT_MAX, None, Some(win_event_hook), 0, 0, 0) };

        let mut msg: MSG = MSG::default();
        loop {
            unsafe {
                if !GetMessageW(&mut msg, HWND(0), 0, 0).as_bool() {
                    log::info!("windows event processing shutdown");
                    break;
                };
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    });
    Ok(())
}
