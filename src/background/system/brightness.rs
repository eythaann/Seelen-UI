use seelen_core::system_state::Brightness;

use crate::{error_handler::Result, windows_api::monitor::Monitor};

#[tauri::command(async)]
pub fn get_main_monitor_brightness() -> Result<Option<Brightness>> {
    let monitor = Monitor::primary();
    let device = monitor.as_monitor_view()?.primary_target()?;

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
    let device = monitor.as_monitor_view()?.primary_target()?;
    device.ioctl_set_display_brightness(brightness.min(100))
}
