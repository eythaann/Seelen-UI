use serde::Serialize;
use windows::Win32::Devices::Display::{
    GetMonitorBrightness, GetMonitorCapabilities, SetMonitorBrightness,
};

use crate::{error_handler::Result, windows_api::WindowsApi};

#[derive(Debug, Serialize)]
pub struct Brightness {
    min: u32,
    max: u32,
    current: u32,
}

#[tauri::command(async)]
pub fn get_main_monitor_brightness() -> Result<Brightness> {
    let mut brightness = Brightness {
        min: 0,
        max: 0,
        current: 0,
    };

    unsafe {
        let hmonitor = WindowsApi::primary_physical_monitor()?;

        let mut pdwmonitorcapabilities: u32 = 0;
        let mut pdwsupportedcolortemperatures: u32 = 0;
        let mut result = GetMonitorCapabilities(
            hmonitor.hPhysicalMonitor,
            &mut pdwmonitorcapabilities,
            &mut pdwsupportedcolortemperatures,
        );

        if result == 0 {
            return Err("GetMonitorCapabilities failed".into());
        }

        result = GetMonitorBrightness(
            hmonitor.hPhysicalMonitor,
            &mut brightness.min,
            &mut brightness.current,
            &mut brightness.max,
        );

        if result == 0 {
            return Err("GetMonitorBrightness failed".into());
        }
    }

    Ok(brightness)
}

#[tauri::command(async)]
pub fn set_main_monitor_brightness(brightness: u32) -> Result<()> {
    let result = unsafe {
        let hmonitor = WindowsApi::primary_physical_monitor()?;
        SetMonitorBrightness(hmonitor.hPhysicalMonitor, brightness)
    };
    if result == 0 {
        return Err("SetMonitorBrightness failed".into());
    }
    Ok(())
}
