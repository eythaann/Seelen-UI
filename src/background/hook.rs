use std::time::Duration;

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, EVENT_MAX, EVENT_MIN,
            EVENT_OBJECT_CLOAKED, EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE,
            EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED, MSG,
        },
    },
};

use crate::{error_handler::Result, seelen::SEELEN, seelenweg::SeelenWeg};

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
        EVENT_OBJECT_DESTROY | EVENT_OBJECT_HIDE | EVENT_OBJECT_CLOAKED => {
            let mut seelen = SEELEN.lock();
            seelen.mut_weg().remove_hwnd(hwnd);
        }
        EVENT_OBJECT_SHOW | EVENT_OBJECT_CREATE | EVENT_OBJECT_UNCLOAKED => {
            if SeelenWeg::should_handle_hwnd(hwnd) {
                let mut seelen = SEELEN.lock();
                seelen.mut_weg().add_hwnd(hwnd);
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
