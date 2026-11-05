use seelen_core::system_state::{Battery, PowerMode, PowerStatus};

use crate::error::Result;

/// OS contract of the power module.
pub trait PowerSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening OS changes, reported through `on_event`.
    /// Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(PowerSysEvent) + Send + Sync + 'static;

    fn power_status(&self) -> Result<PowerStatus>;

    fn batteries(&self) -> Result<Vec<Battery>>;

    /// Whether the OS has updates waiting for a shutdown or restart to be applied.
    fn has_pending_os_updates(&self) -> bool;

    fn log_out(&self) -> Result<()>;

    fn suspend(&self) -> Result<()>;

    fn hibernate(&self) -> Result<()>;

    fn lock(&self) -> Result<()>;

    /// Updates are only applied when explicitly requested (`install_updates`).
    fn restart(&self, install_updates: bool) -> Result<()>;

    /// Updates are only applied when explicitly requested (`install_updates`).
    fn shutdown(&self, install_updates: bool) -> Result<()>;
}

/// Raw OS notifications: they only signal what changed, the manager queries the new data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerSysEvent {
    StatusChanged,
    BatteriesChanged,
    /// The OS doesn't expose a query for the mode, so it comes with the notification.
    ModeChanged {
        mode: PowerMode,
    },
    /// The system resumed from sleep/hibernation.
    Resumed,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum PowerEvent {
    StatusChanged { status: PowerStatus },
    BatteriesChanged { batteries: Vec<Battery> },
    ModeChanged { mode: PowerMode },
}

/// Converts a device read through the cross-platform `battery` crate.
pub fn battery_from_device(battery: battery::Battery) -> Battery {
    let percentage = (battery.state_of_charge().value * 100.0).round();

    Battery {
        vendor: battery.vendor().map(|v| v.to_string()),
        model: battery.model().map(|v| v.to_string()),
        serial_number: battery.serial_number().map(|v| v.to_string()),
        technology: battery.technology().to_string(),

        state: battery.state().to_string(),
        capacity: battery.state_of_health().value,
        temperature: battery.temperature().map(|t| t.value),
        percentage,
        cycle_count: battery.cycle_count(),
        // smart charging means that battery wont be fully charged.
        smart_charging: battery.state() == battery::State::Full && percentage < 100.0,

        energy: battery.energy().value,
        energy_full: battery.energy_full().value,
        energy_full_design: battery.energy_full_design().value,
        energy_rate: battery.energy_rate().value,
        voltage: battery.voltage().value,

        time_to_full: battery.time_to_full().map(|t| t.value),
        time_to_empty: battery.time_to_empty().map(|t| t.value),
    }
}
