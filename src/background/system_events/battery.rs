use serde::Serialize;
use tauri::{AppHandle, Manager, Wry};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

use crate::{error_handler::Result, utils::sleep_millis};

pub fn register_battery_events(handle: AppHandle<Wry>) {
    #[allow(non_snake_case)]
    #[derive(Serialize, Clone)]
    struct Battery {
        pub ACLineStatus: u8,
        pub BatteryFlag: u8,
        pub BatteryLifePercent: u8,
        pub SystemStatusFlag: u8,
        pub BatteryLifeTime: u32,
        pub BatteryFullLifeTime: u32,
    }

    std::thread::spawn(move || -> Result<()> {
        loop {
            let mut power_status = SYSTEM_POWER_STATUS::default();
            unsafe {
                GetSystemPowerStatus(&mut power_status as _)?;
            }
            handle
                .emit("power-status", Battery {
                    ACLineStatus: power_status.ACLineStatus,
                    BatteryFlag: power_status.BatteryFlag,
                    BatteryLifePercent: power_status.BatteryLifePercent,
                    SystemStatusFlag: power_status.SystemStatusFlag,
                    BatteryLifeTime: power_status.BatteryLifeTime,
                    BatteryFullLifeTime: power_status.BatteryFullLifeTime,
                })
                .expect("could not emit event");
            sleep_millis(1000);
        }
    });
}
