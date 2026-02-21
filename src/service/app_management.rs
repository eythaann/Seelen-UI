use slu_ipc::{AppIpc, IPC};
use sysinfo::ProcessesToUpdate;
use windows::Win32::{
    Foundation::HANDLE,
    System::Power::{
        RegisterSuspendResumeNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS, HPOWERNOTIFY,
    },
    UI::WindowsAndMessaging::{DEVICE_NOTIFY_CALLBACK, PBT_APMRESUMESUSPEND},
};

use crate::{enviroment::was_installed_using_msix, error::Result};

/// Starts monitoring the Seelen UI app for the current session
/// Restarts it if it crashes unexpectedly
pub fn start_app_monitoring() {
    std::thread::spawn(move || {
        let mut crash_counter = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));

            if AppIpc::can_stablish_connection() {
                continue;
            }

            log::info!("Seelen UI was closed unexpectedly.");
            crash_counter += 1;
            if crash_counter > 5 {
                break;
            }

            #[cfg(not(debug_assertions))]
            crate::log_error!(launch_seelen_ui());
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    });
}

/// will start the app on the interactive session
pub fn launch_seelen_ui() -> Result<()> {
    let path = if was_installed_using_msix() {
        "shell:AppsFolder\\Seelen.SeelenUI_p6yyn03m1894e!App".to_string()
    } else {
        std::env::current_exe()?
            .with_file_name("seelen-ui.exe")
            .to_string_lossy()
            .to_string()
    };
    // Use explorer.exe to spawn the app de-elevated (with the interactive user's token),
    // since the service runs elevated (TASK_RUNLEVEL_HIGHEST). ShellExecuteExW called
    // from an elevated process inherits the elevated token, causing an infinite restart loop.
    std::process::Command::new("explorer.exe")
        .arg(&path)
        .spawn()?;
    Ok(())
}

pub fn kill_all_seelen_ui_processes() -> Result<()> {
    log::info!("Killing all Seelen UI processes");
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let instances: Vec<_> = sys
        .processes()
        .values()
        .filter(|p| p.exe().is_some_and(|path| path.ends_with("seelen-ui.exe")))
        .collect();
    for instance in instances {
        instance.kill();
    }
    Ok(())
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

/// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registersuspendresumenotification
/// https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nc-powrprof-device_notify_callback_routine
unsafe extern "system" fn power_sleep_resume_proc(
    _context: *const core::ffi::c_void,
    event: u32,
    _setting: *const core::ffi::c_void,
) -> u32 {
    log::debug!("Received power event: {event}");
    if event == PBT_APMRESUMESUSPEND {
        // this probably won't be needed anymore
        // kill_all_seelen_ui_processes().unwrap();
    }
    0
}
