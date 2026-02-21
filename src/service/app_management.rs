use slu_ipc::{AppIpc, IPC};
use sysinfo::ProcessesToUpdate;

use crate::{enviroment::was_installed_using_msix, error::Result, exit};

/// Starts monitoring the Seelen UI app for the current session
/// Restarts it if it crashes unexpectedly
pub fn start_app_monitoring() {
    std::thread::spawn(move || {
        let mut crash_counter = 0;
        let max_tries = if cfg!(debug_assertions) { 0 } else { 5 };
        let mut app_was_connected = false;

        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));

            if crate::EXITING.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }

            if AppIpc::can_stablish_connection() {
                // Reset counter once the app is confirmed running after a restart
                if crash_counter > 0 && !app_was_connected {
                    log::info!("Seelen UI reconnected successfully, resetting crash counter.");
                    crash_counter = 0;
                }
                app_was_connected = true;
                continue;
            }

            // Only count as a crash if we had a confirmed connection before
            if !app_was_connected {
                continue;
            }

            app_was_connected = false;
            log::info!("Seelen UI was closed unexpectedly.");
            crash_counter += 1;
            if crash_counter > max_tries {
                break;
            }

            crate::log_error!(launch_seelen_ui());
            std::thread::sleep(std::time::Duration::from_secs(3));
        }

        log::error!("Seelen UI crashed {crash_counter} times in a row, stopping service.");
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
