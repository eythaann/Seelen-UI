use seelen_core::system_state::{Battery, PowerMode, PowerStatus};
use windows::Win32::System::Power::{EFFECTIVE_POWER_MODE, SYSTEM_POWER_STATUS};

use crate::error::Result;

pub fn power_status_to_serializable(power_status: SYSTEM_POWER_STATUS) -> PowerStatus {
    PowerStatus {
        ac_line_status: power_status.ACLineStatus,
        battery_flag: power_status.BatteryFlag,
        battery_life_percent: power_status.BatteryLifePercent,
        system_status_flag: power_status.SystemStatusFlag,
        battery_life_time: power_status.BatteryLifeTime,
        battery_full_life_time: power_status.BatteryFullLifeTime,
    }
}

pub fn power_mode_to_serializable(mode: EFFECTIVE_POWER_MODE) -> PowerMode {
    match mode.0 {
        0 => PowerMode::BatterySaver,
        1 => PowerMode::BetterBattery,
        2 => PowerMode::Balanced,
        3 => PowerMode::HighPerformance,
        4 => PowerMode::MaxPerformance,
        5 => PowerMode::GameMode,
        6 => PowerMode::MixedReality,
        _ => PowerMode::Unknown,
    }
}

pub fn battery_to_slu_battery(battery: battery::Battery) -> Result<Battery> {

    battery.refresh()?;
    
    let percentage = (battery.state_of_charge().value * 100.0).round();

    Ok(Battery {
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
    })
}
