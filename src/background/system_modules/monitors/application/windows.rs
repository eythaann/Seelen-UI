use std::{sync::Arc, time::Duration};

use seelen_core::system_state::{MonitorId, PhysicalMonitor};
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
    platform::monitor::{DisplayView, Monitor},
    system_modules::monitors::domain::{MonitorsSysEvent, MonitorsSystem},
    windows_api::event_window::subscribe_to_background_window,
};

const SETTLE_ATTEMPTS: u32 = 20;
const SETTLE_RETRY_DELAY: Duration = Duration::from_millis(150);

type EventCallback = Arc<dyn Fn(MonitorsSysEvent) + Send + Sync>;

pub struct WindowsMonitors {
    /// DisplayManager manages critical hardware so be sure to be correctly used, or will make the app crash.
    /// https://learn.microsoft.com/en-us/uwp/api/windows.devices.display.core.displaymanager
    display_manager: Option<DisplayManager>,
    enabled_token: Option<i64>,
    disabled_token: Option<i64>,
    changed_token: Option<i64>,
    paths_failed_or_invalidated_token: Option<i64>,
}

impl MonitorsSystem for WindowsMonitors {
    fn create() -> Self {
        Self {
            display_manager: None,
            enabled_token: None,
            disabled_token: None,
            changed_token: None,
            paths_failed_or_invalidated_token: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(MonitorsSysEvent) + Send + Sync + 'static,
    {
        let on_event: EventCallback = Arc::new(on_event);
        let display_manager = DisplayManager::Create(DisplayManagerOptions::None)?;

        // DisplayManager.Start() requires subscribing to all events first
        // See: https://learn.microsoft.com/en-us/uwp/api/windows.devices.display.core.displaymanager.start
        self.enabled_token = display_manager
            .Enabled(&TypedEventHandler::new(on_enabled))
            .ok();

        self.disabled_token = display_manager
            .Disabled(&TypedEventHandler::new(on_disabled))
            .ok();

        let cb = on_event.clone();
        self.changed_token = display_manager
            .Changed(&TypedEventHandler::new(
                move |_sender: windows_core::Ref<DisplayManager>,
                      args: windows_core::Ref<DisplayManagerChangedEventArgs>| {
                    // this only detects changes on the display adapters like connect/disconnect
                    log::trace!("DisplayManager changed");
                    cb(MonitorsSysEvent::Changed);
                    // Critical!: app will crash if this is not set
                    if let Some(args) = args.as_ref() {
                        args.SetHandled(true)?;
                    }
                    Ok(())
                },
            ))
            .ok();

        let cb = on_event.clone();
        self.paths_failed_or_invalidated_token =
            display_manager
                .PathsFailedOrInvalidated(&TypedEventHandler::new(
                    move |_sender: windows_core::Ref<DisplayManager>,
                          args: windows_core::Ref<
                        DisplayManagerPathsFailedOrInvalidatedEventArgs,
                    >| {
                        log::trace!("DisplayManager paths failed or invalidated");
                        cb(MonitorsSysEvent::Changed);
                        // Critical!: app will crash if this is not set
                        if let Some(args) = args.as_ref() {
                            args.SetHandled(true)?;
                        }
                        Ok(())
                    },
                ))
                .ok();

        display_manager.Start()?;
        self.display_manager = Some(display_manager);

        subscribe_to_background_window(move |event, _w_param, _l_param| {
            if event == WM_DISPLAYCHANGE {
                log::debug!("Displays changed");
                on_event(MonitorsSysEvent::Changed);
            }
            Ok(())
        });
        Ok(())
    }

    fn active_view_ids(&self) -> Result<Vec<MonitorId>> {
        let display_manager = self
            .display_manager
            .as_ref()
            .ok_or("Display manager not initialized")?;
        let state = display_manager
            .TryReadCurrentStateForAllTargets()?
            .State()?;

        let mut ids = Vec::new();
        for view in state.Views()? {
            let view = DisplayView::from(view);
            if !view.is_active().unwrap_or(false) {
                continue;
            }
            if let Ok(id) = view.primary_target().and_then(|t| t.stable_id()) {
                ids.push(id);
            }
        }
        Ok(ids)
    }

    /// Stable-ordered snapshot of the current physical monitors (rect/scale/primary).
    fn physical_monitors(&self) -> Vec<PhysicalMonitor> {
        let mut monitors: Vec<PhysicalMonitor> = Monitor::enumerate()
            .map(|monitors| {
                monitors
                    .into_iter()
                    .filter_map(|m| m.snapshot().ok())
                    .collect()
            })
            .unwrap_or_default();
        monitors.sort_by(|a, b| a.id.0.cmp(&b.id.0));
        monitors
    }

    /// Polls the Win32 monitor enumeration until it reflects `expected_count` monitors
    /// with non-degenerate rects: Windows can take a moment after WinRT reports a target as
    /// connected to finish applying the mode/DPI/work-area for a newly attached monitor.
    fn wait_until_settled(&self, expected_count: usize) {
        for attempt in 0..SETTLE_ATTEMPTS {
            if let Ok(monitors) = Monitor::enumerate() {
                let ready = monitors.len() == expected_count
                    && monitors
                        .iter()
                        .all(|m| m.rect().is_ok_and(|r| r.right > r.left && r.bottom > r.top));
                if ready {
                    return;
                }
            }
            if attempt + 1 < SETTLE_ATTEMPTS {
                std::thread::sleep(SETTLE_RETRY_DELAY);
            }
        }
        log::warn!(
            "Win32 monitor state did not settle after display change, proceeding with possibly stale data"
        );
    }

    fn set_hdr(&self, id: &MonitorId, enabled: bool) -> Result<()> {
        let monitor = Monitor::enumerate()?
            .into_iter()
            .find(|m| matches!(m.get_stable_info(), Ok((mid, _)) if &mid == id))
            .ok_or("Monitor not found")?;
        monitor.set_hdr_state(enabled)
    }
}

impl Drop for WindowsMonitors {
    fn drop(&mut self) {
        let Some(display_manager) = &self.display_manager else {
            return;
        };
        display_manager.Stop().log_error();
        if let Some(token) = self.enabled_token.take() {
            display_manager.RemoveEnabled(token).log_error();
        }
        if let Some(token) = self.disabled_token.take() {
            display_manager.RemoveDisabled(token).log_error();
        }
        if let Some(token) = self.changed_token.take() {
            display_manager.RemoveChanged(token).log_error();
        }
        if let Some(token) = self.paths_failed_or_invalidated_token.take() {
            display_manager
                .RemovePathsFailedOrInvalidated(token)
                .log_error();
        }
    }
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
