#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[allow(non_snake_case)]
pub struct PowerStatus {
    pub ac_line_status: u8,
    pub battery_flag: u8,
    pub battery_life_percent: u8,
    pub system_status_flag: u8,
    pub battery_life_time: u32,
    pub battery_full_life_time: u32,
}

// https://learn.microsoft.com/en-us/windows/win32/api/powersetting/ne-powersetting-effective_power_mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[repr(i32)]
#[ts(repr(enum = name))]
pub enum PowerMode {
    BatterySaver,
    BetterBattery,
    Balanced,
    HighPerformance,
    MaxPerformance,
    GameMode,
    MixedReality,
    Unknown = i32::MAX,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Battery {
    // static info
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub technology: String,
    // common information
    pub state: String,
    pub capacity: f32,
    pub temperature: Option<f32>,
    pub percentage: f32,
    pub cycle_count: Option<u32>,
    pub smart_charging: bool, // this is triggered by windows idk how but this is a simulation of that
    // energy stats
    pub energy: f32,
    pub energy_full: f32,
    pub energy_full_design: f32,
    pub energy_rate: f32,
    pub voltage: f32,
    // charge stats
    pub time_to_full: Option<f32>,
    pub time_to_empty: Option<f32>,
}
