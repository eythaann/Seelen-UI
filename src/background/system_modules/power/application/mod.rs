#[cfg(windows)]
mod windows;

use std::{sync::LazyLock, time::Duration};

use seelen_core::system_state::{Battery, PowerMode, PowerStatus};

use crate::{
    error::{Result, ResultLogExt},
    event_manager, get_tokio_handle,
    utils::lock_free::TracedMutex,
};

use super::domain::{PowerEvent, PowerSysEvent, PowerSystem};

#[cfg(windows)]
type Platform = windows::WindowsPower;

/// The power state may be stale right after waking up, so it is refreshed again after this delay.
const RESUME_REFRESH_DELAY: Duration = Duration::from_secs(2);

pub struct PowerManager {
    platform: Platform,
    state: TracedMutex<PowerState>,
}

event_manager!(PowerManager, PowerEvent);

impl PowerManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<PowerManager> = LazyLock::new(|| {
            let mut m = PowerManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            state: TracedMutex::new(PowerState::default()),
        }
    }

    /// The initial state is read first and independently, so a failing registration (or a
    /// failing battery read) never leaves the rest of the state empty.
    fn init(&mut self) -> Result<()> {
        {
            let mut state = self.state.lock();
            if let Some(status) = self.platform.power_status().ok_logged() {
                state.status = status;
            }
            if let Some(batteries) = self.platform.batteries().ok_logged() {
                state.batteries = batteries;
            }
        }
        self.platform.init(Self::on_platform_event)
    }

    fn on_platform_event(event: PowerSysEvent) {
        let manager = Self::instance();
        match event {
            PowerSysEvent::StatusChanged => manager.refresh_status().log_error(),
            PowerSysEvent::BatteriesChanged => manager.refresh_batteries().log_error(),
            PowerSysEvent::ModeChanged { mode } => manager.update_mode(mode),
            PowerSysEvent::Resumed => {
                get_tokio_handle().spawn(async {
                    tokio::time::sleep(RESUME_REFRESH_DELAY).await;
                    let manager = Self::instance();
                    manager.refresh_status().log_error();
                    manager.refresh_batteries().log_error();
                });
            }
        }
    }

    fn refresh_status(&self) -> Result<()> {
        let status = self.platform.power_status()?;
        let charger_changed = {
            let mut state = self.state.lock();
            if state.status == status {
                return Ok(());
            }
            let charger_changed = state.status.ac_line_status != status.ac_line_status;
            state.status = status.clone();
            charger_changed
        };

        // plugging/unplugging the charger changes the batteries state too
        if charger_changed {
            self.refresh_batteries()?;
        }
        Self::send(PowerEvent::StatusChanged { status });
        Ok(())
    }

    fn refresh_batteries(&self) -> Result<()> {
        let batteries = self.platform.batteries()?;
        self.state.lock().batteries = batteries.clone();
        Self::send(PowerEvent::BatteriesChanged { batteries });
        Ok(())
    }

    fn update_mode(&self, mode: PowerMode) {
        {
            let mut state = self.state.lock();
            if state.mode == mode {
                return;
            }
            state.mode = mode;
        }
        log::trace!("Power mode changed to {mode:?}");
        Self::send(PowerEvent::ModeChanged { mode });
    }

    pub fn status(&self) -> PowerStatus {
        self.state.lock().status.clone()
    }

    pub fn mode(&self) -> PowerMode {
        self.state.lock().mode
    }

    pub fn batteries(&self) -> Vec<Battery> {
        self.state.lock().batteries.clone()
    }

    pub fn has_pending_os_updates(&self) -> bool {
        self.platform.has_pending_os_updates()
    }

    pub fn log_out(&self) -> Result<()> {
        self.platform.log_out()
    }

    pub fn suspend(&self) -> Result<()> {
        self.platform.suspend()
    }

    pub fn hibernate(&self) -> Result<()> {
        self.platform.hibernate()
    }

    pub fn lock(&self) -> Result<()> {
        self.platform.lock()
    }

    pub fn restart(&self, install_updates: bool) -> Result<()> {
        self.platform.restart(install_updates)
    }

    pub fn shutdown(&self, install_updates: bool) -> Result<()> {
        self.platform.shutdown(install_updates)
    }
}

/// Last known values, used to answer queries and to drop notifications without changes.
#[derive(Debug, Default)]
struct PowerState {
    status: PowerStatus,
    mode: PowerMode,
    batteries: Vec<Battery>,
}
