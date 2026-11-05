#[cfg(windows)]
mod windows;

use std::{sync::LazyLock, time::Duration};

use seelen_core::system_state::{MonitorId, PhysicalMonitor};
use slu_utils::{Debounce, debounce};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    system_modules::system_settings::{SystemSettings, SystemSettingsEvent},
    utils::lock_free::SyncVec,
};

use super::domain::{MonitorManagerEvent, MonitorsSysEvent, MonitorsSystem};

#[cfg(windows)]
type Platform = windows::WindowsMonitors;

/// Bursts of display notifications are coalesced by this delay before diffing the state.
const REFRESH_DEBOUNCE: Duration = Duration::from_millis(400);

pub struct MonitorManager {
    platform: Platform,
    /// Stable ids of the active views, used to report views added/removed.
    view_ids: SyncVec<MonitorId>,
    /// Last known monitors, used to report changes that keep the same views.
    monitors: SyncVec<PhysicalMonitor>,
}

event_manager!(MonitorManager, MonitorManagerEvent);

impl MonitorManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<MonitorManager> = LazyLock::new(|| {
            let mut m = MonitorManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            view_ids: SyncVec::new(),
            monitors: SyncVec::new(),
        }
    }

    /// The monitors don't depend on the display listeners, so they are loaded first: a failing
    /// registration must not leave the monitor data (used to deploy the widgets) empty.
    fn init(&mut self) -> Result<()> {
        self.monitors.replace(self.platform.physical_monitors());

        // the scale is part of the monitor data
        SystemSettings::subscribe(|event| {
            if event == SystemSettingsEvent::TextScaleChanged {
                log::debug!("Text scale changed, refreshing monitors");
                Self::request_refresh();
            }
        });

        self.platform.init(|_event: MonitorsSysEvent| {
            Self::request_refresh();
        })?;
        self.view_ids.replace(self.platform.active_view_ids()?);
        Ok(())
    }

    /// Coalesces bursts of display notifications (several of them can fire for a single
    /// physical connect/disconnect) and only notifies real changes.
    fn request_refresh() {
        static DEBOUNCER: LazyLock<Debounce<()>> = LazyLock::new(|| {
            debounce(
                |_| match MonitorManager::instance().refresh() {
                    Ok(true) => MonitorManager::send(MonitorManagerEvent::ViewsChanged),
                    Ok(false) => log::trace!("Display state refresh requested but nothing changed"),
                    Err(e) => log::warn!("Failed to check for display changes: {e}"),
                },
                REFRESH_DEBOUNCE,
            )
        });
        DEBOUNCER.call(());
    }

    /// Diffs the current state against the last known one, sending `ViewAdded`/`ViewRemoved`
    /// for any real differences. Returns whether anything actually changed.
    fn refresh(&self) -> Result<bool> {
        let current_ids = self.platform.active_view_ids()?;
        self.platform.wait_until_settled(current_ids.len());

        let old_ids = self.view_ids.to_vec();
        self.view_ids.replace(current_ids.clone());

        let mut changed = false;
        for id in current_ids.iter().filter(|id| !old_ids.contains(id)) {
            changed = true;
            Self::send(MonitorManagerEvent::ViewAdded { id: id.clone() });
        }
        for id in old_ids.into_iter().filter(|id| !current_ids.contains(id)) {
            changed = true;
            Self::send(MonitorManagerEvent::ViewRemoved { id });
        }

        // the view set can stay identical while rect/scale/primary changed
        // (monitor rearranged, DPI change, primary swapped), so diff the monitors too.
        let monitors = self.platform.physical_monitors();
        if self.monitors.to_vec() != monitors {
            changed = true;
            self.monitors.replace(monitors);
        }
        Ok(changed)
    }

    pub fn get_cached_ids(&self) -> Vec<MonitorId> {
        self.view_ids.to_vec()
    }

    pub fn get_cached_data(&self) -> Vec<PhysicalMonitor> {
        self.monitors.to_vec()
    }

    pub fn set_hdr(&self, id: &MonitorId, enabled: bool) -> Result<()> {
        self.platform.set_hdr(id, enabled)
    }
}
