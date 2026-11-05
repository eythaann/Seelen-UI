use windows::Win32::Devices::Display::{
    DestroyPhysicalMonitor, GetMonitorBrightness, GetMonitorCapabilities, PHYSICAL_MONITOR,
    SetMonitorBrightness,
};

use windows_core::BOOL;

use crate::{
    error::Result,
    platform::monitor::{Monitor, WindowsMonitorExt},
    windows_api::{WindowsApi, string_utils::WindowsString},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DdcciBrightnessValues {
    pub min: u32,
    pub current: u32,
    pub max: u32,
}

/// Owned `PHYSICAL_MONITOR` handle, released with `DestroyPhysicalMonitor` on drop.
///
/// Brightness goes through Display Data Channel / Command Interface (DDC/CI) via `dxva2.dll`.
/// These calls talk to the monitor over the I2C bus of the video cable, so they are slow
/// (tens of milliseconds, seconds on a misbehaving monitor) and must never run on the UI
/// thread. Callers should also serialize them: concurrent DDC/CI transactions to the same
/// monitor can corrupt each other.
pub struct PhysicalMonitorHandle(PHYSICAL_MONITOR);

impl std::fmt::Debug for PhysicalMonitorHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handle = self.0.hPhysicalMonitor;
        f.debug_struct("PhysicalMonitorHandle")
            .field("handle", &handle)
            .field("description", &self.description())
            .finish()
    }
}

unsafe impl Send for PhysicalMonitorHandle {}
unsafe impl Sync for PhysicalMonitorHandle {}

impl Drop for PhysicalMonitorHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyPhysicalMonitor(self.0.hPhysicalMonitor);
        }
    }
}

impl PhysicalMonitorHandle {
    /// Human readable description reported by the driver, e.g. `Generic PnP Monitor`.
    pub fn description(&self) -> String {
        // `PHYSICAL_MONITOR` is packed, so the array has to be copied out before borrowing it
        let description = self.0.szPhysicalMonitorDescription;
        WindowsString::from_slice(&description).to_string()
    }

    /// Whether the monitor claims DDC/CI support through `GetMonitorCapabilities`.
    ///
    /// Only advisory: some monitors that do answer brightness requests fail this call (e.g.
    /// BenQ ZOWIE XL), and Microsoft documents that others report capabilities they don't
    /// actually have. Prefer probing `ddcci_get_brightness` directly and validating its result.
    #[allow(dead_code)]
    pub fn supports_ddcci(&self) -> bool {
        unsafe {
            let mut pdwmonitorcapabilities: u32 = 0;
            let mut pdwsupportedcolortemperatures: u32 = 0;
            // This function fails if the monitor does not support DDC/CI.
            BOOL(GetMonitorCapabilities(
                self.0.hPhysicalMonitor,
                &mut pdwmonitorcapabilities,
                &mut pdwsupportedcolortemperatures,
            ))
            .as_bool()
        }
    }

    pub fn ddcci_get_brightness(&self) -> Result<DdcciBrightnessValues> {
        let mut values = DdcciBrightnessValues::default();
        unsafe {
            BOOL(GetMonitorBrightness(
                self.0.hPhysicalMonitor,
                &mut values.min,
                &mut values.current,
                &mut values.max,
            ))
            .ok()?;
        }
        Ok(values)
    }

    pub fn ddcci_set_brightness(&self, value: u32) -> Result<()> {
        unsafe {
            BOOL(SetMonitorBrightness(self.0.hPhysicalMonitor, value)).ok()?;
        }
        Ok(())
    }
}

/// The physical panels behind this logical monitor. Usually one, but a cloned/mirrored
/// display group shares a single `HMONITOR` and yields one handle per panel.
pub fn physical_monitors(monitor: &Monitor) -> Result<Vec<PhysicalMonitorHandle>> {
    Ok(WindowsApi::get_physical_monitors(monitor.handle())?
        .into_iter()
        .map(PhysicalMonitorHandle)
        .collect())
}
