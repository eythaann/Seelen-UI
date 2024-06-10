use serde::Serialize;
use windows::Win32::System::Power::SYSTEM_POWER_STATUS;

#[allow(non_snake_case)]
#[derive(Serialize, Clone)]
pub struct PowerStatus {
    pub ACLineStatus: u8,
    pub BatteryFlag: u8,
    pub BatteryLifePercent: u8,
    pub SystemStatusFlag: u8,
    pub BatteryLifeTime: u32,
    pub BatteryFullLifeTime: u32,
}

impl From<SYSTEM_POWER_STATUS> for PowerStatus {
    fn from(power_status: SYSTEM_POWER_STATUS) -> Self {
        Self {
            ACLineStatus: power_status.ACLineStatus,
            BatteryFlag: power_status.BatteryFlag,
            BatteryLifePercent: power_status.BatteryLifePercent,
            SystemStatusFlag: power_status.SystemStatusFlag,
            BatteryLifeTime: power_status.BatteryLifeTime,
            BatteryFullLifeTime: power_status.BatteryFullLifeTime,
        }
    }
}
