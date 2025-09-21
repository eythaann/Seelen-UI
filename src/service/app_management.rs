use std::{
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use windows::Win32::{
    Foundation::HANDLE,
    System::Power::{
        RegisterSuspendResumeNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, HPOWERNOTIFY,
    },
    UI::{
        Shell::{FOLDERID_LocalAppData, FOLDERID_Windows},
        WindowsAndMessaging::{DEVICE_NOTIFY_CALLBACK, PBT_APMRESUMESUSPEND},
    },
};

use crate::{
    enviroment::was_installed_using_msix, error::Result, was_started_from_startup_action,
    windows_api::WindowsApi,
};

pub static GUI_RESTARTED_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn launch_seelen_ui() -> Result<()> {
    let explorer = WindowsApi::known_folder(FOLDERID_Windows)?.join("explorer.exe");

    let app_path = if was_installed_using_msix() {
        WindowsApi::known_folder(FOLDERID_LocalAppData)?
            .join("Microsoft\\WindowsApps\\seelen-ui.exe")
    } else {
        std::env::current_exe()?.with_file_name("seelen-ui.exe")
    };

    let mut args = Vec::new();
    if was_started_from_startup_action() {
        args.push("--startup".to_string());
    }

    let lnk_file = WindowsApi::create_temp_shortcut(&app_path, &args.join(" "))?;
    // start it using explorer to spawn it as unelevated
    Command::new(explorer).arg(&lnk_file).status()?;
    std::fs::remove_file(&lnk_file)?;
    Ok(())
}

pub fn kill_seelen_ui_processes() -> Result<()> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    let instances: Vec<_> = sys
        .processes()
        .values()
        .filter(|p| p.exe().is_some_and(|path| path.ends_with("seelen-ui.exe")))
        .collect();
    for instance in instances {
        instance.kill();
    }
    GUI_RESTARTED_COUNTER.store(0, Ordering::SeqCst);
    Ok(())
}

/// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registersuspendresumenotification
/// https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nc-powrprof-device_notify_callback_routine
unsafe extern "system" fn power_sleep_resume_proc(
    _context: *const core::ffi::c_void,
    event: u32,
    _setting: *const core::ffi::c_void,
) -> u32 {
    log::debug!("Received power event: {event}");
    if event == PBT_APMRESUMESUSPEND {
        kill_seelen_ui_processes().unwrap();
    }
    0
}

/// on Dropped will unregister all the handlers
#[allow(dead_code)]
pub struct SystemEventHandlers {
    power: HPOWERNOTIFY,
}

pub fn start_listening_system_events() -> Result<SystemEventHandlers> {
    let mut recipient = DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
        Callback: Some(power_sleep_resume_proc),
        ..Default::default()
    };
    let handler = unsafe {
        RegisterSuspendResumeNotification(
            HANDLE(&mut recipient as *mut _ as _),
            DEVICE_NOTIFY_CALLBACK,
        )
    }?;

    Ok(SystemEventHandlers { power: handler })
}
