use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, MSG, PostQuitMessage,
        RegisterClassW, SW_HIDE, SW_SHOWNORMAL, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE,
        WM_DESTROY, WM_ENDSESSION, WM_QUERYENDSESSION, WNDCLASSW,
    },
};

use crate::{
    error::Result,
    string_utils::WindowsString,
    windows_api::{
        WindowsApi,
        app_bar::{AppBarData, AppBarDataState},
        iterator::WindowEnumerator,
    },
};

/// Tracks whether we actually hid the native taskbar, so we only restore it
/// when it was hidden by us (avoids unnecessary restores on settings changes/shutdown).
static NATIVE_TASKBAR_HIDDEN: AtomicBool = AtomicBool::new(false);
/// Set by every hide request and consumed by the worker before each pass, so a request
/// that arrives while a pass is running is never lost: it schedules one more pass.
static HIDE_REQUESTED: AtomicBool = AtomicBool::new(false);
/// Only one worker thread runs hide passes at a time.
static HIDE_WORKER_RUNNING: AtomicBool = AtomicBool::new(false);
/// Delay before each check of a hide pass. The first checks are close together because explorer
/// usually re-shows the taskbar right after it is hidden; the last ones are spaced out to catch it
/// re-showing later (after its own startup, display changes, etc.). Checks are cheap: they only
/// act on taskbars that are visible or not auto-hide yet.
const HIDE_PASS_DELAYS_MS: [u64; 14] =
    [0, 50, 50, 50, 50, 50, 50, 50, 50, 50, 250, 500, 1000, 2000];

pub fn get_taskbars_handles() -> Result<Vec<HWND>> {
    let mut founds = Vec::new();
    WindowEnumerator::new().for_each(|hwnd| {
        let class = WindowsApi::get_class(hwnd);
        if (class == "Shell_TrayWnd" || class == "Shell_SecondaryTrayWnd")
            && WindowsApi::get_title(hwnd).is_empty()
        {
            founds.push(hwnd);
        }
    })?;
    Ok(founds)
}

pub fn hide_native_taskbar() {
    NATIVE_TASKBAR_HIDDEN.store(true, Ordering::SeqCst);
    HIDE_REQUESTED.store(true, Ordering::SeqCst);
    if HIDE_WORKER_RUNNING.swap(true, Ordering::SeqCst) {
        // the running worker will see HIDE_REQUESTED and run another pass
        return;
    }
    std::thread::spawn(|| {
        loop {
            while HIDE_REQUESTED.swap(false, Ordering::SeqCst) {
                run_hide_pass();
            }
            HIDE_WORKER_RUNNING.store(false, Ordering::SeqCst);
            // A request may have arrived after the last swap but before releasing the worker
            // flag; its caller saw the worker as running and returned, so pick it up here
            // unless another worker already did.
            if !HIDE_REQUESTED.load(Ordering::SeqCst)
                || HIDE_WORKER_RUNNING.swap(true, Ordering::SeqCst)
            {
                break;
            }
        }
    });
}

fn run_hide_pass() {
    let mut focus_moved = false;
    for delay in HIDE_PASS_DELAYS_MS {
        if delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
        if !NATIVE_TASKBAR_HIDDEN.load(Ordering::SeqCst) {
            // restored meanwhile; a later hide request schedules a new pass
            return;
        }
        // Enumerated on every check: taskbars can be created (explorer restart, new monitor)
        // or destroyed while a pass is running.
        let handles = match get_taskbars_handles() {
            Ok(handles) => handles,
            Err(err) => {
                log::error!("Failed to get taskbars handles: {err:?}");
                continue;
            }
        };
        if handles.is_empty() {
            continue;
        }

        // ABM_SETSTATE makes explorer re-run its auto-hide logic and show the taskbar again,
        // which fires a new show event and so a new hide request. On Windows 10 21H2 that code
        // path can also crash explorer, so only send it when the state really has to change.
        // The auto-hide state is shared by all the taskbars, it belongs to the primary one.
        let primary = handles
            .iter()
            .find(|hwnd| WindowsApi::get_class(**hwnd) == "Shell_TrayWnd")
            .unwrap_or(&handles[0]);
        let app_bar = AppBarData::from_handle(*primary);
        if !app_bar.is_auto_hide() {
            app_bar.set_state(AppBarDataState::AutoHide);
        }

        // Explorer keeps a focused taskbar shown, so move the focus to the desktop first.
        // Only once per pass, as it can fall back to simulating an alt tap.
        let foreground = WindowsApi::get_foreground_window();
        if !focus_moved && handles.contains(&foreground) {
            focus_moved = true;
            let desktop = WindowsApi::get_shell_window();
            if !desktop.is_invalid() {
                log::debug!("Native taskbar is focused, moving the focus to the desktop");
                if let Err(err) = WindowsApi::set_foreground(desktop.0 as isize) {
                    log::warn!("Failed to move the focus out of the native taskbar: {err:?}");
                }
            }
        }

        // moving the focus can take a while, don't hide a taskbar restored meanwhile
        if !NATIVE_TASKBAR_HIDDEN.load(Ordering::SeqCst) {
            return;
        }
        for hwnd in &handles {
            if WindowsApi::is_window_visible(*hwnd) {
                log::debug!("Hiding visible native taskbar {:?}", hwnd.0);
                let _ = WindowsApi::show_window_async(hwnd.0 as isize, SW_HIDE.0);
            }
        }
    }
}

pub fn restore_native_taskbar() -> Result<()> {
    if NATIVE_TASKBAR_HIDDEN
        .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Ok(());
    }

    for hwnd in get_taskbars_handles()? {
        AppBarData::from_handle(hwnd).set_state(AppBarDataState::AlwaysOnTop);
        WindowsApi::show_window_async(hwnd.0 as isize, SW_SHOWNORMAL.0)?;
    }
    Ok(())
}

unsafe extern "system" fn shutdown_window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            // sent by the OS when it wants to shut down/log off/quiesce the process
            // (system shutdown, user logoff, or a package update needing us to exit).
            // Reacting here lets us call `exit()` promptly instead of idling in the
            // exit channel wait until the OS times out and force-kills us.
            WM_QUERYENDSESSION | WM_ENDSESSION => {
                log::info!("Received shutdown/quiesce request (msg={msg}), exiting service");
                crate::exit(0);
            }
            WM_DESTROY => PostQuitMessage(0),
            _ => {}
        }
        DefWindowProcW(hwnd, msg, w_param, l_param)
    }
}

/// will lock until the window is closed
unsafe fn create_shutdown_window(done: &crossbeam_channel::Sender<()>) -> Result<()> {
    unsafe {
        let title = WindowsString::from_str("Seelen UI Service Shutdown Window");
        let class = WindowsString::from_str("SeelenServiceShutdownWindow");

        let h_module = GetModuleHandleW(None)?;

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(shutdown_window_proc),
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

        done.send(())?;
        let mut msg = MSG::default();

        // GetMessageW will run until PostQuitMessage(0) is called
        while GetMessageW(&mut msg, Some(hwnd), 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Ok(())
    }
}

/// Spawns a hidden top-level window on its own thread whose only purpose is receiving
/// `WM_QUERYENDSESSION`/`WM_ENDSESSION`, so the service can gracefully exit when the OS
/// tries to quiesce it (shutdown, logoff, package update) instead of being force-killed
/// while idling and reported as a hang.
pub fn start_shutdown_listener() -> Result<()> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    std::thread::Builder::new()
        .name("Shutdown Window".to_string())
        .spawn(move || {
            if let Err(err) = unsafe { create_shutdown_window(&tx) } {
                log::error!("Shutdown window thread failed: {err:?}");
            }
        })?;
    rx.recv()?;
    Ok(())
}
