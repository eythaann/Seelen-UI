use seelen_core::{
    Point,
    rect::Rect,
    system_state::{MonitorId, PhysicalMonitor},
};
use windows::Win32::Graphics::Gdi::{HMONITOR, MONITORINFOEXW};

use crate::{
    error::Result,
    windows_api::{MonitorEnumerator, monitor::Monitor as Win32Monitor},
};

use super::{Monitor, NativeMonitor};

/// Delegates to the `HMONITOR` based implementation of `windows_api`, so the behavior is
/// exactly the same while both types coexist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsMonitor(Win32Monitor);

impl NativeMonitor for WindowsMonitor {
    fn primary() -> Self {
        Self(Win32Monitor::primary())
    }

    fn at_point(point: &Point) -> Self {
        Self(Win32Monitor::at_point(point))
    }

    fn enumerate() -> Result<Vec<Self>> {
        Ok(MonitorEnumerator::enumerate_win32()?
            .into_iter()
            .map(Self)
            .collect())
    }

    fn address(&self) -> isize {
        self.0.handle().0 as isize
    }

    fn is_primary(&self) -> bool {
        self.0.is_primary()
    }

    fn get_stable_info(&self) -> Result<(MonitorId, String)> {
        self.0.get_stable_info()
    }

    fn stable_id(&self) -> Result<MonitorId> {
        self.0.stable_id()
    }

    fn friendly_name(&self) -> Result<String> {
        self.0.friendly_name()
    }

    fn rect(&self) -> Result<Rect> {
        self.0.rect()
    }

    fn work_area(&self) -> Result<Rect> {
        let rect = self.0.info()?.monitorInfo.rcWork;
        Ok(Rect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        })
    }

    fn scale_factor(&self) -> Result<f64> {
        self.0.scale_factor()
    }

    fn hdr_state(&self) -> Result<Option<bool>> {
        self.0.hdr_state()
    }

    fn set_hdr_state(&self, enabled: bool) -> Result<()> {
        self.0.set_hdr_state(enabled)
    }

    fn snapshot(&self) -> Result<PhysicalMonitor> {
        self.0.snapshot()
    }
}

impl From<isize> for WindowsMonitor {
    fn from(address: isize) -> Self {
        Self(Win32Monitor::from(address))
    }
}

/// Windows only concepts of a monitor, use it only from Windows specific code.
pub trait WindowsMonitorExt {
    fn handle(&self) -> HMONITOR;
    fn info(&self) -> Result<MONITORINFOEXW>;
    fn target_device_paths(&self) -> Result<Vec<String>>;
}

impl WindowsMonitorExt for Monitor {
    fn handle(&self) -> HMONITOR {
        self.0.0.handle()
    }

    fn info(&self) -> Result<MONITORINFOEXW> {
        self.0.0.info()
    }

    fn target_device_paths(&self) -> Result<Vec<String>> {
        self.0.0.target_device_paths()
    }
}

// Interop with `windows_api` while the app migrates to the platform layer.

impl From<HMONITOR> for Monitor {
    fn from(hmonitor: HMONITOR) -> Self {
        Self(WindowsMonitor(Win32Monitor::from(hmonitor)))
    }
}

impl From<Win32Monitor> for Monitor {
    fn from(monitor: Win32Monitor) -> Self {
        Self(WindowsMonitor(monitor))
    }
}

impl From<Monitor> for Win32Monitor {
    fn from(monitor: Monitor) -> Self {
        monitor.0.0
    }
}
