use std::{collections::HashMap, sync::LazyLock};

use seelen_core::system_state::MonitorId;
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
    utils::lock_free::SyncHashMap,
    windows_api::{event_window::subscribe_to_background_window, monitor::DisplayView},
};

static MONITOR_MANAGER: LazyLock<MonitorManager> = LazyLock::new(|| {
    let mut m = MonitorManager::create().expect("Failed to create monitor manager");
    m.initialize().log_error();
    m
});

pub struct MonitorManager {
    state_views: SyncHashMap<MonitorId, DisplayView>,
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
            state_views.insert(view.primary_target()?.stable_id()?, view);
        }

        Ok(MonitorManager {
            display_manager,
            state_views: SyncHashMap::from(state_views),
            enabled_token: None,
            disabled_token: None,
            changed_token: None,
            paths_failed_or_invalidated_token: None,
        })
    }

    pub fn instance() -> &'static MonitorManager {
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
        subscribe_to_background_window(|event, _w_param, _l_param| {
            if event == WM_DISPLAYCHANGE {
                log::debug!("Displays changed");
                Self::send(MonitorManagerEvent::ViewsChanged);
                Self::check_for_display_changes().log_error();
            }
            Ok(())
        });
        Ok(())
    }

    // Is recommended that subscribers re-enumerate all targets and state in this call,
    // since the system display stack could be left in any state before this event is raised.
    fn on_enabled(
        _sender: &Option<DisplayManager>,
        args: &Option<DisplayManagerEnabledEventArgs>,
    ) -> windows_core::Result<()> {
        log::trace!("DisplayManager enabled");

        // Critical!: app will crash if this is not set
        if let Some(args) = args {
            args.SetHandled(true)?;
        }
        Ok(())
    }

    // Is recommended that subscribers attempt to clean up when Disabled is invoked.
    // Most display APIs will fail while the session display stack is disabled.
    fn on_disabled(
        _sender: &Option<DisplayManager>,
        args: &Option<DisplayManagerDisabledEventArgs>,
    ) -> windows_core::Result<()> {
        log::trace!("DisplayManager disabled");

        // Critical!: app will crash if this is not set
        if let Some(args) = args {
            args.SetHandled(true)?;
        }
        Ok(())
    }

    // this only detects changes on the display adapters like connect/disconnect of displays
    fn on_changed(
        _sender: &Option<DisplayManager>,
        args: &Option<DisplayManagerChangedEventArgs>,
    ) -> windows_core::Result<()> {
        log::trace!("DisplayManager changed");
        Self::check_for_display_changes().log_error();

        // Critical!: app will crash if this is not set
        if let Some(args) = args {
            args.SetHandled(true)?;
        }
        Ok(())
    }

    fn on_paths_failed_or_invalidated(
        _sender: &Option<DisplayManager>,
        args: &Option<DisplayManagerPathsFailedOrInvalidatedEventArgs>,
    ) -> windows_core::Result<()> {
        log::trace!("DisplayManager paths failed or invalidated");
        // Treat this as a change event
        Self::check_for_display_changes().log_error();

        // Critical!: app will crash if this is not set
        if let Some(args) = args {
            args.SetHandled(true)?;
        }
        Ok(())
    }

    fn check_for_display_changes() -> windows_core::Result<()> {
        let current_state = Self::instance()
            .display_manager
            .TryReadCurrentStateForAllTargets()?
            .State()?;

        let mut current_views = HashMap::new();
        for view in current_state.Views()? {
            let view = DisplayView::from(view);
            let id = match view.primary_target().and_then(|t| t.stable_id()) {
                Ok(id) => id,
                Err(_) => continue,
            };
            current_views.insert(id, view);
        }

        let mut old_views = Self::instance().state_views.to_hash_map();

        // new monitors were added
        for id in current_views.keys() {
            if old_views.remove(id).is_none() {
                Self::send(MonitorManagerEvent::ViewAdded(id.clone()));
            }
        }

        // residuals were removed/disconnected
        for (id, _) in old_views {
            Self::send(MonitorManagerEvent::ViewRemoved(id));
        }

        Self::instance().state_views.replace(current_views);
        Ok(())
    }

    pub fn get_display_view_for_target(&self, target_id: &MonitorId) -> Result<DisplayView> {
        let state = self.display_manager.TryReadCurrentStateForAllTargets()?;
        let state = state.State()?;

        for target in self.display_manager.GetCurrentTargets()? {
            if target.StableMonitorId()?.to_string_lossy() == target_id.0 {
                return Ok(state.GetViewForTarget(&target)?.into());
            }
        }
        Err("Can not find display view for target".into())
    }

    pub fn read_all_views(&self) -> Result<Vec<DisplayView>> {
        let state = self.display_manager.TryReadCurrentStateForAllTargets()?;
        let state = state.State()?;
        Ok(state.Views()?.into_iter().map(DisplayView::from).collect())
    }

    pub fn read_view_at(&self, index: u32) -> Result<DisplayView> {
        let state = self.display_manager.TryReadCurrentStateForAllTargets()?;
        let state = state.State()?;
        let view = state.Views()?.GetAt(index)?;
        Ok(view.into())
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
