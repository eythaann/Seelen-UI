use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::{
        Power::RegisterSuspendResumeNotification,
        RemoteDesktop::{WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION},
    },
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
        RegisterClassW, RegisterShellHookWindow, RegisterWindowMessageW, TranslateMessage,
        DEVICE_NOTIFY_WINDOW_HANDLE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY,
        WM_WTSSESSION_CHANGE, WNDCLASSW, WTS_SESSION_LOCK, WTS_SESSION_LOGOFF, WTS_SESSION_LOGON,
        WTS_SESSION_UNLOCK,
    },
};

use crate::{
    app::emit_to_webviews,
    error::{Result, WindowsResultExt},
    event_manager, log_error,
    utils::spawn_named_thread,
};

use super::{string_utils::WindowsString, WindowsApi};

pub static WM_SHELLHOOKMESSAGE: AtomicU32 = AtomicU32::new(u32::MAX);
pub const HSHELL_FULLSCREEN_ENTER: u32 = 53;
pub const HSHELL_FULLSCREEN_EXIT: u32 = 54;

pub static BACKGROUND_HWND: AtomicIsize = AtomicIsize::new(0);

/// Global flag to track if the current session is interactive (not locked/switched).
/// Used to pause background threads and event processing when the session is not interactive.
pub static IS_INTERACTIVE_SESSION: AtomicBool = AtomicBool::new(true);

pub struct BgWindowProc {}

event_manager!(BgWindowProc, (u32, usize, isize));

impl BgWindowProc {
    /// will lock until the window is closed
    unsafe fn _create_background_window(done: &crossbeam_channel::Sender<()>) -> Result<()> {
        let title = WindowsString::from("Seelen UI Background Window");
        let class = WindowsString::from("SeelenBackgroundWindow");

        let h_module = WindowsApi::module_handle_w()?;

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(Self::window_proc),
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

        let handle: isize = hwnd.0 as isize;
        BACKGROUND_HWND.store(handle, Ordering::Relaxed);

        // register window to recieve shell events
        {
            RegisterShellHookWindow(hwnd).ok().filter_fake_error()?;
            let msg = WindowsString::from("SHELLHOOK");
            WM_SHELLHOOKMESSAGE.store(RegisterWindowMessageW(msg.as_pcwstr()), Ordering::Relaxed);
        }

        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registersuspendresumenotification
        let _resume_suspend_handle =
            RegisterSuspendResumeNotification(hwnd.into(), DEVICE_NOTIFY_WINDOW_HANDLE)?;

        // Register for session change notifications (lock/unlock, user switch, etc.)
        // This is critical for pausing background threads when session is not interactive
        WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION)?;

        done.send(())?;
        let mut msg = MSG::default();

        // GetMessageW will run until PostQuitMessage(0) is called
        while GetMessageW(&mut msg, Some(hwnd), 0, 0).into() {
            TranslateMessage(&msg).ok().filter_fake_error()?;
            DispatchMessageW(&msg);
        }
        Ok(())
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

        // Handle session change notifications to pause background processing
        // when the session is locked or user switches
        if msg == WM_WTSSESSION_CHANGE {
            match w_param.0 as u32 {
                WTS_SESSION_LOCK | WTS_SESSION_LOGOFF => {
                    log::info!("Session locked/logged off - pausing background event processing");
                    IS_INTERACTIVE_SESSION.store(false, Ordering::Release);
                }
                WTS_SESSION_UNLOCK | WTS_SESSION_LOGON => {
                    log::info!("Session unlocked/logged on - resuming background event processing");
                    IS_INTERACTIVE_SESSION.store(true, Ordering::Release);
                    emit_to_webviews("internal::session_resumed", ());
                }
                _ => {}
            }
        }

        Self::send((msg, w_param.0, l_param.0));
        DefWindowProcW(hwnd, msg, w_param, l_param)
    }
}

/// the objective with this window is having a thread that will receive window events
/// and propagate them across the application (common events are keyboard, power, display, etc)
pub fn create_background_window() -> Result<()> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    spawn_named_thread("Background Window", move || {
        log::trace!("Creating background window...");
        log_error!(unsafe { BgWindowProc::_create_background_window(&tx) });
    });
    rx.recv()?;
    log::trace!("Background window created");
    Ok(())
}

pub fn subscribe_to_background_window<F>(callback: F)
where
    F: Fn(u32, usize, isize) -> Result<()> + Send + Sync + 'static,
{
    BgWindowProc::subscribe(move |arg| {
        log_error!(callback(arg.0, arg.1, arg.2));
    });
}
