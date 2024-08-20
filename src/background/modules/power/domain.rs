use serde::Serialize;
use windows::Win32::System::Power::SYSTEM_POWER_STATUS;

use crate::error_handler::{AppError, Result};

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerStatus {
    pub ac_line_status: u8,
    pub battery_flag: u8,
    pub battery_life_percent: u8,
    pub system_status_flag: u8,
    pub battery_life_time: u32,
    pub battery_full_life_time: u32,
}

impl From<SYSTEM_POWER_STATUS> for PowerStatus {
    fn from(power_status: SYSTEM_POWER_STATUS) -> Self {
        Self {
            ac_line_status: power_status.ACLineStatus,
            battery_flag: power_status.BatteryFlag,
            battery_life_percent: power_status.BatteryLifePercent,
            system_status_flag: power_status.SystemStatusFlag,
            battery_life_time: power_status.BatteryLifeTime,
            battery_full_life_time: power_status.BatteryFullLifeTime,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Battery {
    // static info
    vendor: Option<String>,
    model: Option<String>,
    serial_number: Option<String>,
    technology: String,
    // common information
    state: String,
    capacity: f32,
    temperature: Option<f32>,
    percentage: f32,
    cycle_count: Option<u32>,
    smart_charging: bool, // this is triggered by windows idk how but this is a simulation of that
    // energy stats
    energy: f32,
    energy_full: f32,
    energy_full_design: f32,
    energy_rate: f32,
    voltage: f32,
    // charge stats
    time_to_full: Option<f32>,
    time_to_empty: Option<f32>,
}

impl TryFrom<battery::Battery> for Battery {
    type Error = AppError;
    fn try_from(battery: battery::Battery) -> Result<Self> {
        let percentage = (battery.state_of_charge().value * 100.0).round();

        Ok(Self {
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
}
