use windows::Win32::{
    Devices::Display::{
        DestroyPhysicalMonitor, GetMonitorBrightness, GetMonitorCapabilities, PHYSICAL_MONITOR,
        SetMonitorBrightness,
    },
    Foundation::HANDLE,
    Storage::FileSystem::{
        CreateFileW, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    },
};

use windows_core::{BOOL, Owned};

use crate::{
    error::Result,
    windows_api::{WindowsApi, monitor::MonitorTarget, string_utils::WindowsString},
};

use super::Monitor;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DdcciBrightnessValues {
    pub min: u32,
    pub current: u32,
    pub max: u32,
}

/// Owned `PHYSICAL_MONITOR` handle, released with `DestroyPhysicalMonitor` on drop.
struct PhysicalMonitorHandle(PHYSICAL_MONITOR);

impl Drop for PhysicalMonitorHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyPhysicalMonitor(self.0.hPhysicalMonitor);
        }
    }
}

#[allow(dead_code)]
impl MonitorTarget {
    /// Opens and returns a file handle for a display device using its DOS device path.\
    /// These handles are only used for the `DeviceIoControl` API (for internal displays);
    /// a handle can still be returned for external displays, but it should not be used.
    fn get_file_handle(&self) -> Result<Owned<HANDLE>> {
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
        Ok(unsafe { Owned::new(handle) })
    }
}

/// Display Data Channel / Command Interface (DDC/CI) access through `dxva2.dll`.
///
/// These calls talk to the monitor over the I2C bus of the video cable, so they are slow
/// (tens of milliseconds, seconds on a misbehaving monitor) and must never run on the UI
/// thread. Callers should also serialize them: concurrent DDC/CI transactions to the same
/// monitor can corrupt each other.
impl Monitor {
    /// Every physical monitor handle obtained through `GetPhysicalMonitorsFromHMONITOR` must be
    /// released with `DestroyPhysicalMonitor`; the returned guard does that on drop.
    fn main_physical(&self) -> Result<PhysicalMonitorHandle> {
        // wrap every handle first so the ones we don't use are still released
        let physical_monitors: Vec<PhysicalMonitorHandle> =
            WindowsApi::get_physical_monitors(self.handle())?
                .into_iter()
                .map(PhysicalMonitorHandle)
                .collect();
        physical_monitors
            .into_iter()
            .next()
            .ok_or("no physical monitor".into())
    }

    /// Whether the monitor claims DDC/CI support through `GetMonitorCapabilities`.
    ///
    /// Note that this is only advisory: some monitors that do answer brightness requests fail
    /// this call (e.g. BenQ GW-series), and Microsoft documents that others report
    /// capabilities they don't actually have. Prefer probing `ddcci_get_monitor_brightness`
    /// directly and validating its result.
    #[allow(dead_code)]
    pub fn supports_ddcci(&self) -> Result<bool> {
        let physical_monitor = self.main_physical()?;
        let ddcci_is_supported = unsafe {
            let mut pdwmonitorcapabilities: u32 = 0;
            let mut pdwsupportedcolortemperatures: u32 = 0;
            // This function fails if the monitor does not support DDC/CI.
            BOOL(GetMonitorCapabilities(
                physical_monitor.0.hPhysicalMonitor,
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
                physical_monitor.0.hPhysicalMonitor,
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
                physical_monitor.0.hPhysicalMonitor,
                value,
            ))
            .ok()?
        };
        Ok(())
    }
}
