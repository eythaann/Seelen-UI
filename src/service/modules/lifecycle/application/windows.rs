use slu_utils::windows::WindowsString;
use std::{
    ffi::c_void,
    sync::{Arc, OnceLock},
};

use windows::Win32::{
    Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM},
    System::{
        LibraryLoader::GetModuleHandleW,
        Power::{
            DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, HPOWERNOTIFY, RegisterSuspendResumeNotification,
            UnregisterSuspendResumeNotification,
        },
    },
    UI::WindowsAndMessaging::{
        CreateWindowExW, DEVICE_NOTIFY_CALLBACK, DefWindowProcW, DispatchMessageW, GetMessageW,
        MSG, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND, PostQuitMessage,
        RegisterClassW, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_DESTROY, WM_ENDSESSION,
        WM_QUERYENDSESSION, WNDCLASSW,
    },
};

use crate::{
    error::Result,
    log_error,
    modules::lifecycle::domain::{LifecycleEvent, LifecycleSystem},
};

type EventCallback = Box<dyn Fn(LifecycleEvent) + Send + Sync>;

/// Callback of the end session window procedure (one per process, set by `init`).
static END_SESSION_CALLBACK: OnceLock<Arc<EventCallback>> = OnceLock::new();

pub struct WindowsLifecycle {
    /// Context of the suspend/resume callback (`Arc::as_ptr`). It must outlive the registration,
    /// which `Drop` releases before the fields are dropped.
    on_event: Option<Arc<EventCallback>>,
    /// Kept alive as long as the suspend/resume registration.
    power_params: Option<PowerParams>,
    power_registration: Option<HPOWERNOTIFY>,
}

impl LifecycleSystem for WindowsLifecycle {
    fn create() -> Self {
        Self {
            on_event: None,
            power_params: None,
            power_registration: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(LifecycleEvent) + Send + Sync + 'static,
    {
        let on_event: Arc<EventCallback> = Arc::new(Box::new(on_event));
        self.on_event = Some(on_event.clone());

        start_end_session_listener(on_event.clone())?;

        // Heap-allocated so its address stays stable for the lifetime of the registration.
        let params =
            self.power_params
                .insert(PowerParams(Box::new(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
                    Callback: Some(on_suspend_resume),
                    Context: Arc::as_ptr(&on_event) as *mut c_void,
                })));
        let registration = unsafe {
            RegisterSuspendResumeNotification(
                HANDLE(&*params.0 as *const _ as _),
                DEVICE_NOTIFY_CALLBACK,
            )?
        };
        self.power_registration = Some(registration);
        log::info!("Power suspend/resume monitoring active.");
        Ok(())
    }
}

impl Drop for WindowsLifecycle {
    fn drop(&mut self) {
        if let Some(registration) = self.power_registration.take() {
            log_error!(unsafe { UnregisterSuspendResumeNotification(registration) });
        }
    }
}

/// Subscription parameters of the suspend/resume registration.
struct PowerParams(Box<DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS>);

// SAFETY: the only pointer inside is the context, an `Arc<EventCallback>` (`Send + Sync`) that
// lives as long as the registration; the struct is never mutated after its creation.
unsafe impl Send for PowerParams {}
unsafe impl Sync for PowerParams {}

unsafe extern "system" fn on_suspend_resume(
    context: *const c_void,
    event_type: u32,
    _setting: *const c_void,
) -> u32 {
    let Some(on_event) = (unsafe { (context as *const EventCallback).as_ref() }) else {
        return 0;
    };
    match event_type {
        PBT_APMSUSPEND => on_event(LifecycleEvent::Suspending),
        PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => on_event(LifecycleEvent::Resumed),
        _ => {}
    }
    0
}

/// Spawns a hidden top-level window on its own thread whose only purpose is receiving
/// `WM_QUERYENDSESSION`/`WM_ENDSESSION`, so the service can gracefully exit when the OS
/// tries to quiesce it (shutdown, logoff, package update) instead of being force-killed
/// while idling and reported as a hang. Returns once the window exists.
fn start_end_session_listener(on_event: Arc<EventCallback>) -> Result<()> {
    if END_SESSION_CALLBACK.set(on_event).is_err() {
        return Err("End session listener already started".into());
    }
    let (tx, rx) = crossbeam_channel::bounded(1);
    std::thread::Builder::new()
        .name("Shutdown Window".to_string())
        .spawn(move || {
            if let Err(err) = unsafe { create_end_session_window(&tx) } {
                log::error!("Shutdown window thread failed: {err:?}");
            }
        })?;
    rx.recv()?;
    Ok(())
}

/// will lock until the window is closed
unsafe fn create_end_session_window(done: &crossbeam_channel::Sender<()>) -> Result<()> {
    unsafe {
        let title = WindowsString::from_str("Seelen UI Service Shutdown Window");
        let class = WindowsString::from_str("SeelenServiceShutdownWindow");

        let h_module = GetModuleHandleW(None)?;

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(end_session_window_proc),
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

unsafe extern "system" fn end_session_window_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            // sent by the OS when it wants to shut down/log off/quiesce the process
            // (system shutdown, user logoff, or a package update needing us to exit).
            WM_QUERYENDSESSION | WM_ENDSESSION => {
                log::debug!("Received end session message (msg={msg})");
                if let Some(on_event) = END_SESSION_CALLBACK.get() {
                    on_event(LifecycleEvent::EndSession);
                }
            }
            WM_DESTROY => PostQuitMessage(0),
            _ => {}
        }
        DefWindowProcW(hwnd, msg, w_param, l_param)
    }
}
