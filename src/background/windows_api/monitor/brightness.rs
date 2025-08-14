use windows::Win32::{
    Devices::Display::{
        GetMonitorBrightness, GetMonitorCapabilities, SetMonitorBrightness, DISPLAYPOLICY_AC,
        DISPLAYPOLICY_DC, DISPLAY_BRIGHTNESS, IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS,
        IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS, PHYSICAL_MONITOR,
    },
    Foundation::{BOOL, HANDLE},
    Storage::FileSystem::{
        CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    },
    System::IO::DeviceIoControl,
};

use crate::{
    error::Result,
    windows_api::{monitor::MonitorTarget, string_utils::WindowsString, WindowsApi},
};

use super::Monitor;

// Seems to currently be missing from windows crate
const DISPLAYPOLICY_BOTH: u8 = 3;

#[derive(Debug, Default)]
pub struct DdcciBrightnessValues {
    pub min: u32,
    pub current: u32,
    pub max: u32,
}

impl MonitorTarget {
    /// Opens and returns a file handle for a display device using its DOS device path.\
    /// These handles are only used for the `DeviceIoControl` API (for internal displays);
    /// a handle can still be returned for external displays, but it should not be used.
    fn get_file_handle(&self) -> Result<HANDLE> {
        let device_id = self.0.TryGetMonitor()?.DeviceId()?.to_os_string();
        let device_id = WindowsString::from(device_id);

        // This could fail for virtual devices e.g. Remote Desktop sessions - they are not real monitors
        let handle = unsafe {
            CreateFileW(
                device_id.as_pcwstr(),
                (FILE_GENERIC_READ | FILE_GENERIC_WRITE).0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                Default::default(),
                None,
            )?
        };
        Ok(handle)
    }

    /// Input/Output Control. Returns 0-100
    pub fn ioctl_query_display_brightness(&self) -> Result<u8> {
        let display_brightness = unsafe {
            let mut bytes_returned = 0;
            let mut display_brightness = DISPLAY_BRIGHTNESS::default();
            DeviceIoControl(
                self.get_file_handle()?,
                IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS,
                None,
                0,
                Some(&mut display_brightness as *mut _ as *mut _),
                size_of::<DISPLAY_BRIGHTNESS>() as u32,
                Some(&mut bytes_returned),
                None,
            )?;
            display_brightness
        };
        match display_brightness.ucDisplayPolicy as u32 {
            DISPLAYPOLICY_AC => Ok(display_brightness.ucACBrightness),
            DISPLAYPOLICY_DC => Ok(display_brightness.ucDCBrightness),
            _ => Err("Unexpected display policy".into()),
        }
    }

    /// Input/Output Control. Sets 0-100
    pub fn ioctl_set_display_brightness(&self, value: u8) -> Result<()> {
        let mut display_brightness = DISPLAY_BRIGHTNESS {
            ucACBrightness: value,
            ucDCBrightness: value,
            ucDisplayPolicy: DISPLAYPOLICY_BOTH,
        };
        let mut bytes_returned = 0;
        unsafe {
            DeviceIoControl(
                self.get_file_handle()?,
                IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS,
                Some(&mut display_brightness as *mut _ as *mut _),
                size_of::<DISPLAY_BRIGHTNESS>() as u32,
                None,
                0,
                Some(&mut bytes_returned),
                None,
            )?;
        }
        // There is a bug where if the IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS is
        // called immediately after then it won't show the newly updated values
        // Doing a very tiny sleep seems to mitigate this
        std::thread::sleep(std::time::Duration::from_millis(1));
        Ok(())
    }
}

#[allow(dead_code)]
impl Monitor {
    fn main_physical(&self) -> Result<PHYSICAL_MONITOR> {
        let physical_monitors = WindowsApi::get_physical_monitors(self.handle())?;
        let main_physical_monitor = physical_monitors.first().ok_or("no physical monitor")?;
        Ok(*main_physical_monitor)
    }

    pub fn supports_ddcci(&self) -> Result<bool> {
        let physical_monitor = self.main_physical()?;
        let ddcci_is_supported = unsafe {
            let mut pdwmonitorcapabilities: u32 = 0;
            let mut pdwsupportedcolortemperatures: u32 = 0;
            // This function fails if the monitor does not support DDC/CI.
            BOOL(GetMonitorCapabilities(
                physical_monitor.hPhysicalMonitor,
                &mut pdwmonitorcapabilities,
                &mut pdwsupportedcolortemperatures,
            ))
            .as_bool()
        };
        Ok(ddcci_is_supported)
    }

    pub fn ddcci_get_monitor_brightness(&self) -> Result<DdcciBrightnessValues> {
        let physical_monitor = self.main_physical()?;
        let mut values = DdcciBrightnessValues::default();
        unsafe {
            BOOL(GetMonitorBrightness(
                physical_monitor.hPhysicalMonitor,
                &mut values.min,
                &mut values.current,
                &mut values.max,
            ))
            .ok()?;
        }
        Ok(values)
    }

    pub fn ddcci_set_monitor_brightness(&self, value: u32) -> Result<()> {
        let physical_monitor = self.main_physical()?;
        unsafe {
            BOOL(SetMonitorBrightness(
                physical_monitor.hPhysicalMonitor,
                value,
            ))
            .ok()?
        };
        Ok(())
    }
}
