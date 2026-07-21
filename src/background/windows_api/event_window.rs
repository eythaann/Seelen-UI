use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::{
        Power::RegisterSuspendResumeNotification,
        RemoteDesktop::{WTSRegisterSessionNotification, NOTIFY_FOR_ALL_SESSIONS},
    },
    UI::{
        Shell::{
            Common::ITEMIDLIST, SHChangeNotifyEntry, SHChangeNotifyRegister,
            SHGetSpecialFolderLocation, SHCNRF_SOURCE,
        },
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
            RegisterClassW, RegisterShellHookWindow, RegisterWindowMessageW, TranslateMessage,
            DEVICE_NOTIFY_WINDOW_HANDLE, MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_APP, WM_DESTROY,
            WM_WTSSESSION_CHANGE, WNDCLASSW, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
        },
    },
};

use crate::{
    error::{Result, ResultLogExt, WindowsResultExt},
    event_manager,
    utils::spawn_named_thread,
    widgets::manager::WIDGET_MANAGER,
};

use super::{string_utils::WindowsString, WindowsApi};

pub static WM_SHELLHOOKMESSAGE: AtomicU32 = AtomicU32::new(u32::MAX);
// pub const HSHELL_FULLSCREEN_ENTER: u32 = 53;
// pub const HSHELL_FULLSCREEN_EXIT: u32 = 54;

pub static BACKGROUND_HWND: AtomicIsize = AtomicIsize::new(0);

/// Window message sent to the background window when the recycle bin state changes.
pub const WM_TRASH_BIN_NOTIFY: u32 = WM_APP + 100;

/// CSIDL for the Recycle Bin virtual folder.
const CSIDL_BITBUCKET: i32 = 0x000a;
/// SHCNRF_ShellLevel
const SHCNRF_SHELL_LEVEL: SHCNRF_SOURCE = SHCNRF_SOURCE(0x0002);
/// SHCNE_ALLEVENTS
const SHCNE_ALL_EVENTS: i32 = 0x7FFFFFFF_u32 as i32;

/// Global flag to track if the current session is interactive (not locked/switched).
/// Used to pause background threads and event processing when the session is not interactive.
pub static IS_INTERACTIVE_SESSION: AtomicBool = AtomicBool::new(true);
pub static IS_LOCKED: AtomicBool = AtomicBool::new(false);

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
        WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_ALL_SESSIONS)?;

        // Register for shell change notifications so that WM_TRASH_BIN_NOTIFY is sent
        // to the background window whenever the recycle bin contents change.
        register_shell_notifications(hwnd);

        done.send(())?;
        let mut msg = MSG::default();

        // GetMessageW will run until PostQuitMessage(0) is called
        while GetMessageW(&mut msg, Some(hwnd), 0, 0).into() {
            TranslateMessage(&msg).ok().filter_fake_error()?;
            DispatchMessageW(&msg);
        }
        Ok(())
    }

    fn refresh_interactive_state() {
        let Some(active_session) = slu_ipc::app::current_interactive_session_id() else {
            // no active session, assume lock screen
            IS_INTERACTIVE_SESSION.store(false, Ordering::Release);
            return;
        };

        let my_session = slu_ipc::app::current_session_id().unwrap_or(u32::MAX);
        let is_interactive = active_session == my_session && !IS_LOCKED.load(Ordering::Acquire);

        log::info!(
            "Active session: {}, my session: {}, is interactive: {}",
            active_session,
            my_session,
            is_interactive
        );

        let previous = IS_INTERACTIVE_SESSION.swap(is_interactive, Ordering::AcqRel);
        if !previous && is_interactive {
            // reload all pods webviews on session resume
            std::thread::spawn(move || {
                WIDGET_MANAGER.deployments.for_each(|(_, deployment)| {
                    deployment.pods.for_each(|(_, pod)| {
                        pod.soft_restart();
                    });
                });
            });
        }
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
            log::trace!(
                "Session change event received: w_param={}, l_param={}",
                w_param.0,
                l_param.0
            );

            match w_param.0 as u32 {
                WTS_SESSION_LOCK
                    if l_param.0 as u32
                        == slu_ipc::app::current_session_id().unwrap_or(u32::MAX) =>
                {
                    IS_LOCKED.store(true, Ordering::Release);
                    log::info!("Session locked");
                }
                WTS_SESSION_UNLOCK
                    if l_param.0 as u32
                        == slu_ipc::app::current_session_id().unwrap_or(u32::MAX) =>
                {
                    IS_LOCKED.store(false, Ordering::Release);
                    log::info!("Session unlocked");
                }
                _ => {}
            }

            Self::refresh_interactive_state();
        }

        Self::send((msg, w_param.0, l_param.0));
        DefWindowProcW(hwnd, msg, w_param, l_param)
    }
}

/// Registers shell change notifications for the Recycle Bin on the background window.
/// When the recycle bin contents change, Windows sends `WM_TRASH_BIN_NOTIFY` to `hwnd`,
/// which is then broadcast to all `subscribe_to_background_window` subscribers.
unsafe fn register_shell_notifications(hwnd: HWND) {
    let pidl: *mut ITEMIDLIST =
        SHGetSpecialFolderLocation(None, CSIDL_BITBUCKET).unwrap_or(std::ptr::null_mut());

    let entry = SHChangeNotifyEntry {
        pidl,
        fRecursive: true.into(),
    };

    SHChangeNotifyRegister(
        hwnd,
        SHCNRF_SHELL_LEVEL,
        SHCNE_ALL_EVENTS,
        WM_TRASH_BIN_NOTIFY,
        1,
        &entry,
    );
}

/// the objective with this window is having a thread that will receive window events
/// and propagate them across the application (common events are keyboard, power, display, etc)
pub fn create_background_window() -> Result<()> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    spawn_named_thread("Background Window", move || {
        log::trace!("Creating background window...");
        unsafe { BgWindowProc::_create_background_window(&tx) }.log_error();
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
        callback(arg.0, arg.1, arg.2).log_error();
    });
}
