use std::{collections::HashMap, sync::LazyLock, time::Duration};

use seelen_core::system_state::{MonitorId, PhysicalMonitor};
use slu_utils::{Debounce, debounce};
use windows::{
    Devices::Display::Core::{
        DisplayManager, DisplayManagerChangedEventArgs, DisplayManagerDisabledEventArgs,
        DisplayManagerEnabledEventArgs, DisplayManagerOptions,
        DisplayManagerPathsFailedOrInvalidatedEventArgs,
    },
    Foundation::TypedEventHandler,
    Win32::UI::WindowsAndMessaging::WM_DISPLAYCHANGE,
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    modules::system_settings::application::{SystemSettings, SystemSettingsEvent},
    utils::lock_free::{SyncHashMap, SyncVec},
    windows_api::{
        MonitorEnumerator, event_window::subscribe_to_background_window, monitor::DisplayView,
    },
};

/// Builds a stable-ordered snapshot of the current physical monitors (rect/scale/primary),
/// used to detect real changes vs spurious WM_DISPLAYCHANGE/WinRT notifications that carry
/// no actual diff (e.g. the same set of targets, but re-reported without any property change).
fn snapshot_physical_monitors() -> Vec<PhysicalMonitor> {
    let mut monitors: Vec<PhysicalMonitor> = MonitorEnumerator::enumerate_win32()
        .map(|monitors| {
            monitors
                .into_iter()
                .filter_map(|m| PhysicalMonitor::try_from(m).ok())
                .collect()
        })
        .unwrap_or_default();
    monitors.sort_by(|a, b| a.id.0.cmp(&b.id.0));
    monitors
}

pub struct MonitorManager {
    state_views: SyncHashMap<MonitorId, DisplayView>,
    state_data: SyncVec<PhysicalMonitor>,
    /// DisplayManager manages critical hardware so be sure to be correctly used, or will make the app crash.
    /// https://learn.microsoft.com/en-us/uwp/api/windows.devices.display.core.displaymanager
    display_manager: DisplayManager,
    enabled_token: Option<i64>,
    disabled_token: Option<i64>,
    changed_token: Option<i64>,
    paths_failed_or_invalidated_token: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorManagerEvent {
    /// the id used is the view primary target id
    ViewAdded(MonitorId),
    /// the id used is the view primary target id
    ViewRemoved(MonitorId),
    ViewsChanged,
}

event_manager!(MonitorManager, MonitorManagerEvent);

impl MonitorManager {
    fn create() -> Result<MonitorManager> {
        let display_manager = DisplayManager::Create(DisplayManagerOptions::None)?;
        let state = display_manager
            .TryReadCurrentStateForAllTargets()?
            .State()?;

        let mut state_views = HashMap::new();
        for view in state.Views()? {
            let view = DisplayView::from(view);
            if !view.is_active()? {
                continue;
            }
            state_views.insert(view.primary_target()?.stable_id()?, view);
        }

        Ok(MonitorManager {
            display_manager,
            state_views: SyncHashMap::from(state_views),
            state_data: SyncVec::from(snapshot_physical_monitors()),
            enabled_token: None,
            disabled_token: None,
            changed_token: None,
            paths_failed_or_invalidated_token: None,
        })
    }

    pub fn instance() -> &'static MonitorManager {
        static MONITOR_MANAGER: LazyLock<MonitorManager> = LazyLock::new(|| {
            let mut m = MonitorManager::create().expect("Failed to create monitor manager");
            m.initialize().log_error();
            m
        });
        &MONITOR_MANAGER
    }

    fn initialize(&mut self) -> Result<()> {
        // DisplayManager.Start() requires subscribing to all events first
        // See: https://learn.microsoft.com/en-us/uwp/api/windows.devices.display.core.displaymanager.start
        self.enabled_token = self
            .display_manager
            .Enabled(&TypedEventHandler::new(Self::on_enabled))
            .ok();

        self.disabled_token = self
            .display_manager
            .Disabled(&TypedEventHandler::new(Self::on_disabled))
            .ok();

        self.changed_token = self
            .display_manager
            .Changed(&TypedEventHandler::new(Self::on_changed))
            .ok();

        self.paths_failed_or_invalidated_token = self
            .display_manager
            .PathsFailedOrInvalidated(&TypedEventHandler::new(
                Self::on_paths_failed_or_invalidated,
            ))
            .ok();

        self.display_manager.Start()?;
        SystemSettings::subscribe(|event| {
            if event == SystemSettingsEvent::TextScaleChanged {
                log::debug!("Text scale changed, re-emitting ViewsChanged");
                Self::request_display_state_refresh();
            }
        });

        subscribe_to_background_window(|event, _w_param, _l_param| {
            if event == WM_DISPLAYCHANGE {
                log::debug!("Displays changed");
                Self::request_display_state_refresh();
            }
            Ok(())
        });
        Ok(())
    }

    /// Coalesces bursts of display notifications (WM_DISPLAYCHANGE + the several
    /// WinRT DisplayManager events can all fire for a single physical connect/disconnect)
    /// and waits for the Win32 topology to settle before diffing state, since Windows
    /// can take a moment after WinRT reports a target as connected to finish applying
    /// the mode/DPI/work-area for a newly attached monitor.
    fn request_display_state_refresh() {
        static DEBOUNCER: LazyLock<Debounce<()>> = LazyLock::new(|| {
            debounce(
                |_| match MonitorManager::check_for_display_changes() {
                    Ok(changed) => {
                        if changed {
                            MonitorManager::send(MonitorManagerEvent::ViewsChanged);
                        } else {
                            log::trace!("Display state refresh requested but nothing changed");
                        }
                    }
                    Err(e) => log::warn!("Failed to check for display changes: {e}"),
                },
                Duration::from_millis(400),
            )
        });
        DEBOUNCER.call(());
    }

    /// Polls the Win32 monitor enumeration until it reflects `expected_count` monitors
    /// with non-degenerate rects, or gives up after a few attempts. Runs on the debounce's
    /// own background thread, so blocking here is safe.
    fn wait_for_win32_to_settle(expected_count: usize) {
        const MAX_ATTEMPTS: u32 = 20;
        const RETRY_DELAY: Duration = Duration::from_millis(150);

        for attempt in 0..MAX_ATTEMPTS {
            if let Ok(monitors) = MonitorEnumerator::enumerate_win32() {
                let ready = monitors.len() == expected_count
                    && monitors
                        .iter()
                        .all(|m| m.rect().is_ok_and(|r| r.right > r.left && r.bottom > r.top));
                if ready {
                    return;
                }
            }
            if attempt + 1 < MAX_ATTEMPTS {
                std::thread::sleep(RETRY_DELAY);
            }
        }
        log::warn!(
            "Win32 monitor state did not settle after display change, proceeding with possibly stale data"
        );
    }

    // Is recommended that subscribers re-enumerate all targets and state in this call,
    // since the system display stack could be left in any state before this event is raised.
    fn on_enabled(
        _sender: windows_core::Ref<DisplayManager>,
        args: windows_core::Ref<DisplayManagerEnabledEventArgs>,
    ) -> windows_core::Result<()> {
        log::trace!("DisplayManager enabled");

        // Critical!: app will crash if this is not set
        if let Some(args) = args.as_ref() {
            args.SetHandled(true)?;
        }
        Ok(())
    }

    // Is recommended that subscribers attempt to clean up when Disabled is invoked.
    // Most display APIs will fail while the session display stack is disabled.
    fn on_disabled(
        _sender: windows_core::Ref<DisplayManager>,
        args: windows_core::Ref<DisplayManagerDisabledEventArgs>,
    ) -> windows_core::Result<()> {
        log::trace!("DisplayManager disabled");

        // Critical!: app will crash if this is not set
        if let Some(args) = args.as_ref() {
            args.SetHandled(true)?;
        }
        Ok(())
    }

    // this only detects changes on the display adapters like connect/disconnect of displays
    fn on_changed(
        _sender: windows_core::Ref<DisplayManager>,
        args: windows_core::Ref<DisplayManagerChangedEventArgs>,
    ) -> windows_core::Result<()> {
        log::trace!("DisplayManager changed");
        Self::request_display_state_refresh();

        // Critical!: app will crash if this is not set
        if let Some(args) = args.as_ref() {
            args.SetHandled(true)?;
        }
        Ok(())
    }

    fn on_paths_failed_or_invalidated(
        _sender: windows_core::Ref<DisplayManager>,
        args: windows_core::Ref<DisplayManagerPathsFailedOrInvalidatedEventArgs>,
    ) -> windows_core::Result<()> {
        log::trace!("DisplayManager paths failed or invalidated");
        // Treat this as a change event
        Self::request_display_state_refresh();

        // Critical!: app will crash if this is not set
        if let Some(args) = args.as_ref() {
            args.SetHandled(true)?;
        }
        Ok(())
    }

    /// Diffs the current WinRT display state against the last known one, emitting
    /// `ViewAdded`/`ViewRemoved` for any real differences. Returns whether anything
    /// actually changed, so callers can avoid reacting to spurious notifications
    /// (WM_DISPLAYCHANGE and the WinRT display events can fire with no real topology change).
    fn check_for_display_changes() -> windows_core::Result<bool> {
        let current_state = Self::instance()
            .display_manager
            .TryReadCurrentStateForAllTargets()?
            .State()?;

        let mut current_views = HashMap::new();
        for view in current_state.Views()? {
            let view = DisplayView::from(view);
            if !view.is_active().unwrap_or(false) {
                continue;
            }
            let id = match view.primary_target().and_then(|t| t.stable_id()) {
                Ok(id) => id,
                Err(_) => continue,
            };
            current_views.insert(id, view);
        }

        Self::wait_for_win32_to_settle(current_views.len());

        let mut old_views = Self::instance().state_views.to_hash_map();
        let current_ids: Vec<MonitorId> = current_views.keys().cloned().collect();
        Self::instance().state_views.replace(current_views);

        let mut changed = false;

        // new monitors were added
        for id in current_ids {
            if old_views.remove(&id).is_none() {
                changed = true;
                Self::send(MonitorManagerEvent::ViewAdded(id.clone()));
            }
        }

        // residuals were removed/disconnected
        for (id, _) in old_views {
            changed = true;
            Self::send(MonitorManagerEvent::ViewRemoved(id));
        }

        // the WinRT target set can stay identical while rect/scale/primary changed
        // (monitor rearranged, DPI change, primary swapped), so diff the physical
        // properties too instead of relying only on target add/remove.
        let physical_snapshot = snapshot_physical_monitors();
        let state_data = &Self::instance().state_data;
        if state_data.to_vec() != physical_snapshot {
            changed = true;
            state_data.replace(physical_snapshot);
        }

        Ok(changed)
    }

    pub fn get_cached_ids(&self) -> Vec<MonitorId> {
        self.state_views.keys()
    }

    pub fn get_cached_data(&self) -> Vec<PhysicalMonitor> {
        self.state_data.to_vec()
    }
}

impl Drop for MonitorManager {
    fn drop(&mut self) {
        self.display_manager.Stop().log_error();

        if let Some(enabled_token) = self.enabled_token {
            self.display_manager
                .RemoveEnabled(enabled_token)
                .log_error();
        }

        if let Some(disabled_token) = self.disabled_token {
            self.display_manager
                .RemoveDisabled(disabled_token)
                .log_error();
        }

        if let Some(changed_token) = self.changed_token {
            self.display_manager
                .RemoveChanged(changed_token)
                .log_error();
        }

        if let Some(paths_failed_or_invalidated_token) = self.paths_failed_or_invalidated_token {
            self.display_manager
                .RemovePathsFailedOrInvalidated(paths_failed_or_invalidated_token)
                .log_error();
        }
    }
}
