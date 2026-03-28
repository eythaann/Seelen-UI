use std::time::{Duration, Instant};

use slu_ipc::{AppIpc, IPC};
use sysinfo::ProcessesToUpdate;

use crate::{enviroment::was_installed_using_msix, error::Result, exit};

/// How long to wait for the app to establish its IPC connection after each launch attempt.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(2);

/// Max consecutive failures before giving up and stopping the service.
/// In debug builds we stop immediately after the first failure.
const MAX_CRASHES: u32 = if cfg!(debug_assertions) { 0 } else { 5 };

/// Starts monitoring the Seelen UI app for the current session and restarts it automatically
/// when it crashes or fails to start.
///
/// `app_just_launched` should be `true` when the caller already launched the app so that the
/// monitor gives it a startup grace window before counting a failure.
pub fn start_app_monitoring(app_just_launched: bool) {
    std::thread::spawn(move || {
        let mut crash_count = 0u32;
        let mut app_was_connected = true;
        // Track when we last launched the app so we can detect startup timeouts.
        let mut launch_time: Option<Instant> = app_just_launched.then(Instant::now);

        loop {
            std::thread::sleep(Duration::from_secs(1));

            if crate::EXITING.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }

            if AppIpc::can_stablish_connection() {
                if !app_was_connected {
                    log::info!("Seelen UI reconnected successfully, resetting crash counter.");
                    crash_count = 0;
                    launch_time = None;
                }
                app_was_connected = true;
                continue;
            }

            // ── App is not reachable via IPC ──────────────────────────────────────────

            // If we recently launched the app, give it time to establish its IPC connection
            // before counting the absence as a failure.
            if matches!(launch_time, Some(lt) if lt.elapsed() < STARTUP_TIMEOUT) {
                continue;
            }

            // Failure: either the app was connected and just disappeared (runtime crash),
            // or a launch attempt's grace window expired without establishing IPC.
            app_was_connected = false;
            crash_count += 1;
            log::warn!("Seelen UI is not running (failure #{crash_count}/{MAX_CRASHES}).");

            if crash_count > MAX_CRASHES {
                log::error!("Seelen UI failed {crash_count} times in a row, stopping service.");
                break;
            }

            crate::log_error!(kill_all_seelen_ui_processes());

            // Linear back-off: 500 ms, 1000 ms, 1500 ms, …
            let backoff = Duration::from_millis(500 * crash_count as u64);
            log::info!("Restarting Seelen UI in {}ms…", backoff.as_millis());
            std::thread::sleep(backoff);
            crate::log_error!(launch_seelen_ui());
            launch_time = Some(Instant::now());
        }

        log::error!("Seelen UI monitoring stopped after repeated failures.");
        exit(1);
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
    log::info!("Killing all Seelen UI processes in current session");
    let current_session = crate::windows_api::WindowsApi::current_session_id();

    let mut sys = sysinfo::System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let instances: Vec<_> = sys
        .processes()
        .values()
        .filter(|p| {
            if !p.exe().is_some_and(|path| path.ends_with("seelen-ui.exe")) {
                return false;
            }
            let mut session_id = 0u32;
            let in_session = unsafe {
                windows::Win32::System::RemoteDesktop::ProcessIdToSessionId(
                    p.pid().as_u32(),
                    &mut session_id,
                )
                .is_ok()
            };
            in_session && session_id == current_session
        })
        .collect();
    for instance in instances {
        instance.kill();
    }
    Ok(())
}
