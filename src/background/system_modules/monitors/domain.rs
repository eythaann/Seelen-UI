use seelen_core::system_state::{MonitorId, PhysicalMonitor};

use crate::error::Result;

/// OS contract of the monitors (display topology) module.
pub trait MonitorsSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening display changes, reported through `on_event`. The OS may notify changes
    /// that leave the topology untouched: diffing them is up to the caller.
    /// Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(MonitorsSysEvent) + Send + Sync + 'static;

    /// Stable ids of the active views (a cloned group is a single view).
    fn active_view_ids(&self) -> Result<Vec<MonitorId>>;

    /// Connected monitors, sorted by id. Brightness is left empty.
    fn physical_monitors(&self) -> Vec<PhysicalMonitor>;

    /// Blocks until the OS reports `expected_count` monitors with a valid layout, or gives up
    /// after a while: a newly connected monitor takes a moment to get its mode/DPI/work area.
    fn wait_until_settled(&self, expected_count: usize);

    fn set_hdr(&self, id: &MonitorId, enabled: bool) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorsSysEvent {
    Changed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorManagerEvent {
    /// the id used is the view primary target id
    ViewAdded {
        id: MonitorId,
    },
    /// the id used is the view primary target id
    ViewRemoved {
        id: MonitorId,
    },
    ViewsChanged,
}
