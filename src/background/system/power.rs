use serde::Serialize;
use tauri::Manager;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::FALSE,
        Security::{
            AdjustTokenPrivileges, SE_PRIVILEGE_ENABLED, SE_SHUTDOWN_NAME, TOKEN_PRIVILEGES,
        },
        System::{
            Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS},
            Shutdown::{EWX_LOGOFF, EWX_REBOOT, EWX_SHUTDOWN, SHTDN_REASON_NONE},
        },
    },
};

use crate::{
    error_handler::{log_if_error, Result},
    seelen::get_app_handle,
    utils::sleep_millis,
    windows_api::WindowsApi,
};

pub fn register_battery_events() {
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
        let handle = get_app_handle();

        loop {
            let mut power_status = SYSTEM_POWER_STATUS::default();
            unsafe {
                GetSystemPowerStatus(&mut power_status as _)?;
            }
            handle
                .emit(
                    "power-status",
                    Battery {
                        ACLineStatus: power_status.ACLineStatus,
                        BatteryFlag: power_status.BatteryFlag,
                        BatteryLifePercent: power_status.BatteryLifePercent,
                        SystemStatusFlag: power_status.SystemStatusFlag,
                        BatteryLifeTime: power_status.BatteryLifeTime,
                        BatteryFullLifeTime: power_status.BatteryFullLifeTime,
                    },
                )
                .expect("could not emit event");
            sleep_millis(1000);
        }
    });
}

#[tauri::command]
pub fn log_out() {
    log_if_error(WindowsApi::exit_windows(EWX_LOGOFF, SHTDN_REASON_NONE));
}

#[tauri::command]
pub fn sleep() {
    log_if_error(WindowsApi::set_suspend_state());
}

#[tauri::command]
pub fn restart() -> Result<(), String> {
    let token_handle = WindowsApi::open_process_token()?;
    let mut tkp = TOKEN_PRIVILEGES::default();

    tkp.PrivilegeCount = 1;
    tkp.Privileges[0].Luid = WindowsApi::get_luid(PCWSTR::null(), SE_SHUTDOWN_NAME)?;
    tkp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

    unsafe {
        AdjustTokenPrivileges(token_handle, FALSE, Some(&tkp), 0, None, None)
            .expect("Could not adjust token privileges");
    }

    WindowsApi::exit_windows(EWX_REBOOT, SHTDN_REASON_NONE)?;
    Ok(())
}

#[tauri::command]
pub fn shutdown() -> Result<(), String> {
    let token_handle = WindowsApi::open_process_token()?;
    let mut tkp = TOKEN_PRIVILEGES::default();

    tkp.PrivilegeCount = 1;
    tkp.Privileges[0].Luid = WindowsApi::get_luid(PCWSTR::null(), SE_SHUTDOWN_NAME)?;
    tkp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

    unsafe {
        AdjustTokenPrivileges(token_handle, FALSE, Some(&tkp), 0, None, None)
            .expect("Could not adjust token privileges");
    }

    WindowsApi::exit_windows(EWX_SHUTDOWN, SHTDN_REASON_NONE)?;
    Ok(())
}
