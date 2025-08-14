use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Brightness, PhysicalMonitor},
};
use tauri::Emitter;

use crate::{
    app::get_app_handle,
    error_handler::Result,
    log_error,
    modules::monitors::MONITOR_MANAGER,
    windows_api::{monitor::Monitor, MonitorEnumerator},
};

use super::MonitorManager;

pub fn register_monitor_webview_events() -> Result<()> {
    MONITOR_MANAGER.lock().init()?;
    MonitorManager::subscribe(|_event| {
        if let Ok(monitors) = get_connected_monitors() {
            log_error!(get_app_handle().emit(SeelenEvent::SystemMonitorsChanged, monitors));
        }
    });
    Ok(())
}

#[tauri::command(async)]
pub fn get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    let mut monitors = Vec::new();
    for m in MonitorEnumerator::get_all_v2()? {
        monitors.push(m.try_into()?);
    }
    Ok(monitors)
}

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
