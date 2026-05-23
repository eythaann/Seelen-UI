use std::ffi::c_void;
use std::sync::atomic::Ordering;

use windows::Win32::{
    Foundation::HANDLE,
    System::Power::{RegisterSuspendResumeNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS},
    UI::WindowsAndMessaging::{
        DEVICE_NOTIFY_CALLBACK, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND,
    },
};

use crate::app_management::{on_system_resume, on_system_suspend};

unsafe extern "system" fn power_callback(
    _context: *const c_void,
    event_type: u32,
    _setting: *const c_void,
) -> u32 {
    match event_type {
        PBT_APMSUSPEND => on_system_suspend(),
        PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => on_system_resume(),
        _ => {}
    }
    0
}

pub fn start_power_monitoring() {
    std::thread::spawn(|| {
        // Heap-allocate params so its address stays stable for the lifetime of the registration.
        let params = Box::new(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
            Callback: Some(power_callback),
            Context: std::ptr::null_mut(),
        });

        let token = unsafe {
            RegisterSuspendResumeNotification(
                HANDLE(&*params as *const _ as _),
                DEVICE_NOTIFY_CALLBACK,
            )
        };

        match token {
            Err(e) => {
                log::error!("Failed to register power notifications: {e:?}");
            }
            Ok(_token) => {
                log::info!("Power suspend/resume monitoring active.");
                // Keep params and _token alive until the service exits.
                while !crate::EXITING.load(Ordering::SeqCst) {
                    std::thread::sleep(std::time::Duration::from_secs(10));
                }
            }
        }
    });
}
