mod brightness;

use itertools::Itertools;
use windows::{
    Devices::Display::Core::{DisplayTarget, DisplayView as WinRTDisplayView},
    Win32::Graphics::Gdi::HMONITOR,
};

use crate::{error::Result, modules::monitors::MonitorManager};
use seelen_core::{rect::Rect, system_state::MonitorId};

use super::{MonitorEnumerator, WindowsApi};

/// This struct represents a screen, a screen could be shown in multiple display devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Monitor(HMONITOR);

unsafe impl Send for Monitor {}
unsafe impl Sync for Monitor {}

impl From<HMONITOR> for Monitor {
    fn from(hmonitor: HMONITOR) -> Self {
        Self(hmonitor)
    }
}

impl From<isize> for Monitor {
    fn from(hmonitor: isize) -> Self {
        Self(HMONITOR(hmonitor as _))
    }
}

impl From<&seelen_core::Point> for Monitor {
    fn from(point: &seelen_core::Point) -> Self {
        let hmonitor = WindowsApi::monitor_from_point(point);
        Self(hmonitor)
    }
}

// HMONITOR on win32 is the same concept as DisplayView in winrt

impl Monitor {
    pub fn handle(&self) -> HMONITOR {
        self.0
    }

    pub fn index(&self) -> Result<usize> {
        let monitors = MonitorEnumerator::enumerate_win32()?;
        let (idx, _) = monitors
            .into_iter()
            .find_position(|monitor| monitor == self)
            .ok_or("Invalid or expired monitor handle")?;
        Ok(idx)
    }

    pub fn primary() -> Monitor {
        Monitor(WindowsApi::primary_monitor())
    }

    pub fn is_primary(&self) -> bool {
        self.0 == WindowsApi::primary_monitor()
    }

    pub fn as_monitor_view(&self) -> Result<DisplayView> {
        MonitorManager::instance().read_view_at(self.index()? as u32)
    }

    pub fn name(&self) -> Result<String> {
        self.as_monitor_view()?.primary_target()?.name()
    }

    pub fn stable_id(&self) -> Result<String> {
        Ok(self.as_monitor_view()?.primary_target()?.stable_id()?.0)
    }

    pub fn stable_id2(&self) -> Result<MonitorId> {
        self.as_monitor_view()?.primary_target()?.stable_id()
    }

    pub fn rect(&self) -> Result<Rect> {
        let rect = WindowsApi::monitor_info(self.0)?.monitorInfo.rcMonitor;
        Ok(Rect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        })
    }

    pub fn scale_factor(&self) -> Result<f64> {
        let monitor_scale_factor = WindowsApi::get_monitor_scale_factor(self.0)?;
        let text_scale_factor = WindowsApi::get_text_scale_factor()?;
        Ok(monitor_scale_factor * text_scale_factor)
    }
}

/// represents a display screen view (one view can be shown in multiple displays 'mirrors')
///
/// # Safety
/// this is unsafe of store for long sessions as this object could be invalid on display changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayView(WinRTDisplayView);

impl DisplayView {
    fn index(&self) -> Result<usize> {
        let searching_id = self.primary_target()?.stable_id()?;
        for (idx, view) in MonitorManager::instance()
            .read_all_views()?
            .into_iter()
            .enumerate()
        {
            let current_id = view.primary_target()?.stable_id()?;
            if current_id == searching_id {
                return Ok(idx);
            }
        }
        Err("Invalid or expired monitor view".into())
    }

    pub fn as_win32_monitor(&self) -> Result<Monitor> {
        let win32_views = MonitorEnumerator::enumerate_win32()?;
        let monitor = win32_views.get(self.index()?).copied();
        Ok(monitor.ok_or("Invalid or expired monitor view")?)
    }

    pub fn targets(&self) -> Result<Vec<MonitorTarget>> {
        let mut targets = Vec::new();
        for path in self.0.Paths()? {
            targets.push(path.Target()?.into());
        }
        Ok(targets)
    }

    pub fn primary_target(&self) -> Result<MonitorTarget> {
        Ok(self.0.Paths()?.GetAt(0)?.Target()?.into())
    }
}

impl From<WinRTDisplayView> for DisplayView {
    fn from(view: WinRTDisplayView) -> Self {
        Self(view)
    }
}

/// represents a physical screen/monitor
pub struct MonitorTarget(DisplayTarget);

impl MonitorTarget {
    pub fn stable_id(&self) -> Result<MonitorId> {
        Ok(self.0.StableMonitorId()?.to_string().into())
    }

    pub fn name(&self) -> Result<String> {
        Ok(self.0.TryGetMonitor()?.DisplayName()?.to_string())
    }
}

impl From<DisplayTarget> for MonitorTarget {
    fn from(target: DisplayTarget) -> Self {
        Self(target)
    }
}
