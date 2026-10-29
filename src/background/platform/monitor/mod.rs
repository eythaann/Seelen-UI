#[cfg(windows)]
mod windows;

use seelen_core::{
    Point,
    rect::Rect,
    system_state::{MonitorId, PhysicalMonitor},
};

use crate::error::Result;

#[cfg(windows)]
pub use windows::WindowsMonitorExt;

#[cfg(windows)]
type Native = windows::WindowsMonitor;

/// A screen surface of the virtual desktop. A screen can be shown in multiple display devices
/// (mirroring), so this is not a physical monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monitor(Native);

impl Monitor {
    pub fn primary() -> Self {
        Self(Native::primary())
    }

    pub fn at_point(point: &Point) -> Self {
        Self(Native::at_point(point))
    }

    pub fn enumerate() -> Result<Vec<Self>> {
        Ok(Native::enumerate()?.into_iter().map(Self).collect())
    }

    /// Native handle as a plain number, useful to send it over IPC.
    pub fn address(&self) -> isize {
        self.0.address()
    }

    pub fn is_primary(&self) -> bool {
        self.0.is_primary()
    }

    /// Returns (stable id, friendly name) in a single query.
    pub fn get_stable_info(&self) -> Result<(MonitorId, String)> {
        self.0.get_stable_info()
    }

    /// Identifier that persists across reboots and reconnections.
    pub fn stable_id(&self) -> Result<MonitorId> {
        self.0.stable_id()
    }

    pub fn friendly_name(&self) -> Result<String> {
        self.0.friendly_name()
    }

    /// Full area of the monitor on the virtual desktop.
    pub fn rect(&self) -> Result<Rect> {
        self.0.rect()
    }

    /// Area of the monitor not covered by OS reserved surfaces (taskbars, docks, panels).
    pub fn work_area(&self) -> Result<Rect> {
        self.0.work_area()
    }

    /// DPI scale of the monitor multiplied by the text scale accessibility setting.
    pub fn scale_factor(&self) -> Result<f64> {
        self.0.scale_factor()
    }

    /// `None` means the monitor doesn't support HDR at all.
    pub fn hdr_state(&self) -> Result<Option<bool>> {
        self.0.hdr_state()
    }

    pub fn set_hdr_state(&self, enabled: bool) -> Result<()> {
        self.0.set_hdr_state(enabled)
    }

    /// Brightness is left empty, it is tracked apart by `BrightnessManager`.
    pub fn snapshot(&self) -> Result<PhysicalMonitor> {
        self.0.snapshot()
    }
}

/// Restores a monitor from a value previously returned by [`Monitor::address`].
impl From<isize> for Monitor {
    fn from(address: isize) -> Self {
        Self(Native::from(address))
    }
}

impl From<&Point> for Monitor {
    fn from(point: &Point) -> Self {
        Self::at_point(point)
    }
}

/// Contract to be implemented by each OS for its native monitor handle.
trait NativeMonitor: Sized + Copy + Eq + Send + Sync + From<isize> {
    fn primary() -> Self;
    fn at_point(point: &Point) -> Self;
    fn enumerate() -> Result<Vec<Self>>;

    fn address(&self) -> isize;
    fn is_primary(&self) -> bool;
    fn get_stable_info(&self) -> Result<(MonitorId, String)>;
    fn stable_id(&self) -> Result<MonitorId>;
    fn friendly_name(&self) -> Result<String>;
    fn rect(&self) -> Result<Rect>;
    fn work_area(&self) -> Result<Rect>;
    fn scale_factor(&self) -> Result<f64>;
    fn hdr_state(&self) -> Result<Option<bool>>;
    fn set_hdr_state(&self, enabled: bool) -> Result<()>;
    fn snapshot(&self) -> Result<PhysicalMonitor>;
}
