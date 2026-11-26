#[cfg(windows)]
mod windows;

use std::{
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use slu_ipc::{AppIpc, IPC};

use crate::{error::Result, exit, log_error};

use super::domain::AppManagementSystem;

#[cfg(windows)]
type Platform = windows::WindowsAppManagement;

/// How long to wait for the app to establish its IPC connection after each launch attempt.
const STARTUP_TIMEOUT: Duration = Duration::from_secs(2);

/// How long to wait for the app to restore its IPC pipe after a system resume.
/// Tauri's async runtime can take several seconds to fully re-initialize.
const RESUME_GRACE: Duration = Duration::from_secs(10);

/// Max consecutive failures before giving up and stopping the service.
/// In debug builds we stop immediately after the first failure.
const MAX_CRASHES: u32 = if cfg!(debug_assertions) { 0 } else { 5 };

pub struct AppManager {
    platform: Platform,
    /// Set to `true` while the system is suspended; cleared on resume.
    /// Written by the lifecycle listener, read by the app monitoring thread.
    system_suspended: AtomicBool,
}

impl AppManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<AppManager> = LazyLock::new(AppManager::create);
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            system_suspended: AtomicBool::new(false),
        }
    }

    /// will start the app on the interactive session
    pub fn launch_app(&self) -> Result<()> {
        self.platform.launch_app()
    }

    pub fn kill_app_instances(&self) -> Result<()> {
        log::info!("Killing all Seelen UI processes in current session");
        self.platform.kill_app_instances()
    }

    pub fn on_system_suspend(&self) {
        log::info!("System suspending — pausing app health checks.");
        self.system_suspended.store(true, Ordering::SeqCst);
    }

    pub fn on_system_resume(&self) {
        log::info!("System resuming — will restore app health checks after grace window.");
        self.system_suspended.store(false, Ordering::SeqCst);
    }

    /// Starts monitoring the Seelen UI app for the current session and restarts it automatically
    /// when it crashes or fails to start.
    ///
    /// `app_just_launched` should be `true` when the caller already launched the app so that the
    /// monitor gives it a startup grace window before counting a failure.
    pub fn start_monitoring(&'static self, app_just_launched: bool) {
        std::thread::spawn(move || self.monitoring_loop(app_just_launched));
    }

    fn monitoring_loop(&self, app_just_launched: bool) {
        let mut crash_count = 0u32;
        let mut app_was_connected = true;
        // Absolute deadline until which IPC failures are ignored (startup / resume grace).
        let mut grace_until: Option<Instant> =
            app_just_launched.then(|| Instant::now() + STARTUP_TIMEOUT);
        let mut was_suspended = false;

        loop {
            std::thread::sleep(Duration::from_secs(1));

            if crate::EXITING.load(Ordering::SeqCst) {
                return;
            }

            // ── Suspend / resume handling ─────────────────────────────────────────────

            if self.system_suspended.load(Ordering::SeqCst) {
                was_suspended = true;
                continue;
            }

            if was_suspended {
                // First iteration after waking up: reset failure state and grant a generous
                // grace window so the app has time to restore its IPC pipe.
                was_suspended = false;
                crash_count = 0;
                app_was_connected = true;
                grace_until = Some(Instant::now() + RESUME_GRACE);
                log::info!(
                    "Resumed from sleep — crash counter reset, granting {}s grace window.",
                    RESUME_GRACE.as_secs()
                );
                continue;
            }

            // ── Normal health check ───────────────────────────────────────────────────

            if AppIpc::can_stablish_connection() {
                if !app_was_connected {
                    log::info!("Seelen UI reconnected successfully, resetting crash counter.");
                    crash_count = 0;
                    grace_until = None;
                }
                app_was_connected = true;
                continue;
            }

            // ── App is not reachable via IPC ──────────────────────────────────────────

            // If we are still within a grace window (post-launch or post-resume), do not
            // count the absence as a failure yet.
            if matches!(grace_until, Some(deadline) if Instant::now() < deadline) {
                continue;
            }

            // Failure: either the app was connected and just disappeared (runtime crash),
            // or a grace window expired without the app establishing IPC.
            app_was_connected = false;
            crash_count += 1;
            log::warn!("Seelen UI is not running (failure #{crash_count}/{MAX_CRASHES}).");

            if crash_count > MAX_CRASHES {
                log::error!("Seelen UI failed {crash_count} times in a row, stopping service.");
                break;
            }

            log_error!(self.kill_app_instances());

            // Linear back-off: 500 ms, 1000 ms, 1500 ms, …
            let backoff = Duration::from_millis(500 * crash_count as u64);
            log::info!("Restarting Seelen UI in {}ms…", backoff.as_millis());
            std::thread::sleep(backoff);
            log_error!(self.launch_app());
            grace_until = Some(Instant::now() + STARTUP_TIMEOUT);
        }

        log::error!("Seelen UI monitoring stopped after repeated failures.");
        exit(1);
    }
}
