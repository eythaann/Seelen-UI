mod brightness;

use itertools::Itertools;
use windows::{
    Devices::Display::Core::{DisplayTarget, DisplayView},
    Win32::Graphics::Gdi::HMONITOR,
};

use crate::{
    error_handler::Result,
    modules::{
        input::domain::Point,
        monitors::{MonitorManager, GLOBAL_DISPLAY_MANAGER},
    },
};
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

impl From<&Point> for Monitor {
    fn from(point: &Point) -> Self {
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
        let monitors = MonitorEnumerator::get_all_v2()?;
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

    pub fn as_monitor_view(&self) -> Result<MonitorView> {
        MonitorManager::view_at(self.index()? as u32)
    }

    pub fn name(&self) -> Result<String> {
        self.as_monitor_view()?.primary_target()?.name()
    }

    pub fn stable_id(&self) -> Result<String> {
        self.as_monitor_view()?.primary_target()?.stable_id()
    }

    pub fn stable_id2(&self) -> Result<MonitorId> {
        self.as_monitor_view()?.primary_target()?.stable_id2()
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

    #[allow(dead_code)]
    pub fn scale_factor(&self) -> Result<f64> {
        let monitor_scale_factor = WindowsApi::get_monitor_scale_factor(self.0)?;
        let text_scale_factor = WindowsApi::get_text_scale_factor()?;
        Ok(monitor_scale_factor * text_scale_factor)
    }
}

/// represents a display screen view (one view can be shown in multiple displays 'mirrors')
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorView(DisplayView);

impl MonitorView {
    pub fn index(&self) -> Result<usize> {
        let state = GLOBAL_DISPLAY_MANAGER.TryReadCurrentStateForModeQuery()?;
        let state = state.State()?;

        let id = self.primary_target()?.stable_id()?;

        for (idx, view) in state.Views()?.into_iter().enumerate() {
            let primary_path = view.Paths()?.GetAt(0)?;
            let primary_target = primary_path.Target()?;
            let pt_id = primary_target.StableMonitorId()?.to_string();

            if pt_id == id {
                return Ok(idx);
            }
        }
        Err("Invalid or expired monitor view".into())
    }

    pub fn as_win32_monitor(&self) -> Result<Monitor> {
        let win32_views = MonitorEnumerator::get_all_v2()?;
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

impl From<DisplayView> for MonitorView {
    fn from(view: DisplayView) -> Self {
        Self(view)
    }
}

/// represents a physical screen/monitor
pub struct MonitorTarget(DisplayTarget);

impl MonitorTarget {
    pub fn stable_id(&self) -> Result<String> {
        Ok(self.0.StableMonitorId()?.to_string())
    }

    pub fn stable_id2(&self) -> Result<MonitorId> {
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
