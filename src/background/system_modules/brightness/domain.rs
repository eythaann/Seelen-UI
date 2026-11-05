use seelen_core::system_state::MonitorId;

use crate::error::Result;

/// OS contract of the monitors brightness module.
pub trait BrightnessSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Looks for the panels of the connected monitors and starts tracking their brightness,
    /// reporting changes through `on_event`.
    /// Called once; the tracking lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(BrightnessEvent) + Send + Sync + 'static;

    /// Looks again for the panels, as monitors were connected/disconnected. Doesn't block.
    fn rescan(&self);

    /// Brightness of the monitor as a percentage, `None` if it can't be controlled.
    fn brightness(&self, id: &MonitorId) -> Option<u8>;

    /// Sets the brightness (`0..=100`) of every panel behind the monitor.
    fn set_brightness(&self, id: &MonitorId, percent: u8) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum BrightnessEvent {
    Changed,
}
