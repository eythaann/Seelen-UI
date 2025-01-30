use serde::Serialize;

use crate::{error_handler::Result, windows_api::monitor::Monitor};

#[derive(Debug, Serialize)]
pub struct Brightness {
    min: u32,
    max: u32,
    current: u32,
}

#[tauri::command(async)]
pub fn get_main_monitor_brightness() -> Result<Option<Brightness>> {
    let monitor = Monitor::primary();
    let device = monitor.main_display_device()?;
    let current = device.ioctl_query_display_brightness()?;
    if current == 0 && device.ioctl_set_display_brightness(0).is_err() {
        return Ok(None);
    }
    Ok(Some(Brightness {
        min: 0,
        max: 100,
        current: current as u32,
    }))
}

#[tauri::command(async)]
pub fn set_main_monitor_brightness(brightness: u8) -> Result<()> {
    let monitor = Monitor::primary();
    let device = monitor.main_display_device()?;
    device.ioctl_set_display_brightness(brightness.min(100))?;
    Ok(())
}
