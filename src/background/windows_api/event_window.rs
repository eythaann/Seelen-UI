use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::sync::Arc;
use windows::Win32::{
    Devices::Display::GUID_DEVINTERFACE_MONITOR,
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
        RegisterClassW, RegisterDeviceNotificationW, TranslateMessage, DBT_DEVTYP_DEVICEINTERFACE,
        DEVICE_NOTIFY_WINDOW_HANDLE, DEV_BROADCAST_DEVICEINTERFACE_W, MSG, WINDOW_EX_STYLE,
        WINDOW_STYLE, WM_DESTROY, WNDCLASSW,
    },
};

use crate::{
    error_handler::{Result, WindowsResultExt},
    log_error, trace_lock,
    utils::spawn_named_thread,
};

use super::{string_utils::WindowsString, WindowsApi};

type Callback = Box<dyn Fn(u32, usize, isize) -> Result<()> + Send + Sync + 'static>;

lazy_static! {
    static ref CALLBACKS: Arc<Mutex<Vec<Callback>>> = Arc::new(Mutex::new(Vec::new()));
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    for callback in CALLBACKS.lock().iter() {
        log_error!(callback(msg, w_param.0, l_param.0));
    }
    DefWindowProcW(hwnd, msg, w_param, l_param)
}

unsafe fn _create_background_window() -> Result<()> {
    let title = WindowsString::from("Seelen UI Event Window");
    let class = WindowsString::from("SeelenUIEventWindow");

    let h_module = WindowsApi::module_handle_w()?;

    let wnd_class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: h_module.into(),
        lpszClassName: class.as_pcwstr(),
        ..Default::default()
    };

    RegisterClassW(&wnd_class);

    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        class.as_pcwstr(),
        title.as_pcwstr(),
        WINDOW_STYLE::default(),
        0,
        0,
        0,
        0,
        None,
        None,
        Some(wnd_class.hInstance),
        None,
    )?;

    // register window to recieve device notifications for monitor changes
    {
        let mut notification_filter = DEV_BROADCAST_DEVICEINTERFACE_W {
            dbcc_size: std::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as u32,
            dbcc_devicetype: DBT_DEVTYP_DEVICEINTERFACE.0,
            dbcc_reserved: 0,
            dbcc_classguid: GUID_DEVINTERFACE_MONITOR,
            dbcc_name: [0; 1],
        };
        RegisterDeviceNotificationW(
            hwnd.into(),
            &mut notification_filter as *mut _ as *mut _,
            DEVICE_NOTIFY_WINDOW_HANDLE,
        )?;
    }

    let mut msg = MSG::default();
    // GetMessageW will run until PostQuitMessage(0) is called
    while GetMessageW(&mut msg, Some(hwnd), 0, 0).into() {
        TranslateMessage(&msg).ok().filter_fake_error()?;
        DispatchMessageW(&msg);
    }
    Ok(())
}

/// the objective with this window is having a thread that will receive window events
/// and propagate them across the application (common events are keyboard, power, display, etc)
pub fn create_background_window() -> Result<std::thread::JoinHandle<()>> {
    spawn_named_thread("Background Window", || {
        log_error!(unsafe { _create_background_window() });
    })
}

pub fn subscribe_to_background_window<F>(callback: F)
where
    F: Fn(u32, usize, isize) -> Result<()> + Send + Sync + 'static,
{
    trace_lock!(CALLBACKS).push(Box::new(callback));
}
