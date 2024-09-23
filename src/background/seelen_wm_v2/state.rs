use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use itertools::Itertools;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::state::{NoFallbackBehavior, WManagerLayoutInfo, WmNode};

use crate::{
    error_handler::Result,
    modules::{input::domain::Point, virtual_desk::get_vd_manager},
    state::application::FULL_STATE,
    windows_api::{monitor::Monitor, window::Window, MonitorEnumerator, WindowsApi},
};

use super::{
    cli::{Axis, Sizing},
    node_impl::WmNodeImpl,
};

lazy_static! {
    pub static ref WM_STATE: Arc<Mutex<WmV2State>> = Arc::new(Mutex::new({
        let mut state = WmV2State::default();
        state
            .init()
            .expect("Failed to initialize Window Manager State");
        state
    }));
}

#[derive(Debug)]
pub struct WmV2StateWorkspace {
    root: Option<WmNodeImpl>,
    layout_info: Option<WManagerLayoutInfo>,
    no_fallback_behavior: NoFallbackBehavior,
}

#[derive(Debug, Default)]
pub struct WmV2StateMonitor {
    pub id: String,
    pub workspaces: HashMap<String, WmV2StateWorkspace>,
}

#[derive(Debug, Default)]
pub struct WmV2State {
    pub monitors: HashMap<String, WmV2StateMonitor>,
}

impl WmV2StateWorkspace {
    pub fn new(monitor_idx: usize, workspace_idx: usize) -> Self {
        let mut workspace = Self {
            layout_info: None,
            root: None,
            no_fallback_behavior: NoFallbackBehavior::Float,
        };

        let settings = FULL_STATE.load();
        let layout_id = settings.get_wm_layout_id(monitor_idx, workspace_idx);
        if let Some(l) = settings.layouts.get(&layout_id).cloned() {
            workspace.layout_info = Some(l.info);
            workspace.root = Some(WmNodeImpl::new(l.structure));
            workspace.no_fallback_behavior = l.no_fallback_behavior;
        }

        workspace
    }

    pub fn get_root_node(&self) -> Option<&WmNode> {
        self.root.as_ref().map(|n| n.inner())
    }

    pub fn add_window(&mut self, window: &Window) {
        if let Some(node) = &mut self.root {
            let residual = node.try_add_window(window);
            if !residual.is_empty() {
                log::warn!("Current Layout is full, and fallback container was not found");
                // TODO
            }
        }
    }

    pub fn remove_window(&mut self, window: &Window) {
        if let Some(node) = &mut self.root {
            let residual = node.remove_window(window);
            if !residual.is_empty() {
                log::warn!("Current Layout is full, and fallback container was not found");
                // TODO
            }
        }
    }

    pub fn contains(&self, window: &Window) -> bool {
        self.root.as_ref().map_or(false, |n| n.contains(window))
    }

    pub fn trace_to(&self, window: &Window) -> Vec<&WmNode> {
        self.root.as_ref().map_or(vec![], |n| n.trace(window))
    }

    pub fn get_node_at_point(&mut self, point: &Point) -> Option<&mut WmNode> {
        if let Some(root) = &mut self.root {
            return root.get_node_at_point(point).ok()?;
        }
        None
    }
}

impl WmV2StateMonitor {
    pub fn create_workspace(monitor_idx: usize, workspace_id: &str) -> Result<WmV2StateWorkspace> {
        for (workspace_idx, w) in get_vd_manager().get_all()?.iter().enumerate() {
            if w.id() == workspace_id {
                return Ok(WmV2StateWorkspace::new(monitor_idx, workspace_idx));
            }
        }
        Err("Invalid workspace id".into())
    }

    pub fn get_workspace_mut(&mut self, workspace_id: &str) -> &mut WmV2StateWorkspace {
        match self.workspaces.entry(workspace_id.to_string()) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                let monitor_idx = Monitor::by_id(&self.id)
                    .expect("Failed to get monitor")
                    .index()
                    .expect("Failed to get monitor index");
                let new_workspace = Self::create_workspace(monitor_idx, workspace_id)
                    .expect("Failed to ensure workspace");
                e.insert(new_workspace)
            }
        }
    }

    pub fn contains(&self, window: &Window) -> bool {
        self.workspaces.values().any(|w| w.contains(window))
    }

    pub fn trace_to(&self, window: &Window) -> Option<(&WmV2StateWorkspace, Vec<&WmNode>)> {
        for w in self.workspaces.values() {
            let trace = w.trace_to(window);
            if !trace.is_empty() {
                return Some((w, trace));
            }
        }
        None
    }
}

impl WmV2State {
    /// will enumarate all monitors and workspaces
    pub fn init(&mut self) -> Result<()> {
        let workspaces = get_vd_manager().get_all()?;
        for (monitor_idx, hmonitor) in MonitorEnumerator::get_all()?.into_iter().enumerate() {
            let id = WindowsApi::monitor_name(hmonitor)?;
            if self.monitors.contains_key(&id) {
                continue;
            }

            let mut monitor = WmV2StateMonitor::default();
            for (workspace_idx, w) in workspaces.iter().enumerate() {
                if monitor.workspaces.contains_key(&w.id()) {
                    continue;
                }

                monitor
                    .workspaces
                    .insert(w.id(), WmV2StateWorkspace::new(monitor_idx, workspace_idx));
            }

            monitor.id = id.clone();
            self.monitors.insert(id, monitor);
        }
        Ok(())
    }

    pub fn get_monitor_mut(&mut self, monitor_id: &str) -> Option<&mut WmV2StateMonitor> {
        self.monitors.get_mut(monitor_id)
    }

    pub fn contains(&self, window: &Window) -> bool {
        self.trace_to(window).is_some()
    }

    pub fn trace_to(
        &self,
        window: &Window,
    ) -> Option<(&WmV2StateMonitor, &WmV2StateWorkspace, Vec<&WmNode>)> {
        for m in self.monitors.values() {
            if let Some((w, trace)) = m.trace_to(window) {
                return Some((m, w, trace));
            }
        }
        None
    }

    pub fn get_node_at_point(&mut self, point: &Point) -> Option<&mut WmNode> {
        let monitor = Monitor::from(point);
        if let Some(m) = self.monitors.get_mut(&monitor.id().ok()?) {
            let current_workspace = get_vd_manager().get_current().ok()?.id();
            if let Some(w) = m.workspaces.get_mut(&current_workspace) {
                return w.get_node_at_point(point);
            }
        }
        None
    }

    pub fn update_size(
        &self,
        window: &Window,
        axis: Axis,
        action: Sizing,
        percentage: f32,
    ) -> Result<(&WmV2StateMonitor, &WmV2StateWorkspace)> {
        if let Some((m, w, trace)) = self.trace_to(window) {
            let valid_parents = trace
                .into_iter()
                .rev()
                .filter(|n| match n {
                    WmNode::Horizontal(inner) => {
                        axis == Axis::Width
                            && inner
                                .children
                                .iter()
                                .filter(|n| n.len() > 0)
                                .collect_vec()
                                .len()
                                >= 2
                    }
                    WmNode::Vertical(inner) => {
                        axis == Axis::Height
                            && inner
                                .children
                                .iter()
                                .filter(|n| n.len() > 0)
                                .collect_vec()
                                .len()
                                >= 2
                    }
                    _ => false,
                })
                .collect_vec();

            let update_sizes = |children: &Vec<WmNode>| {
                let total_pai: f32 = children.iter().map(|n| n.grow_factor().get()).sum();

                let to_grow = total_pai * percentage / 100f32;
                let to_reduce = to_grow / (children.len() - 1) as f32;

                for n in children {
                    if WmNodeImpl::new((*n).clone()).contains(window) {
                        n.grow_factor().set(match action {
                            Sizing::Increase => n.grow_factor().get() + to_grow,
                            Sizing::Decrease => n.grow_factor().get() - to_grow,
                        });
                    } else {
                        n.grow_factor().set(match action {
                            Sizing::Increase => n.grow_factor().get() - to_reduce,
                            Sizing::Decrease => n.grow_factor().get() + to_reduce,
                        });
                    }
                }
            };

            match valid_parents.first() {
                Some(WmNode::Vertical(inner)) => update_sizes(&inner.children),
                Some(WmNode::Horizontal(inner)) => update_sizes(&inner.children),
                _ => log::warn!("Can't change width if the window is alone"),
            }
            return Ok((m, w));
        }
        Err("Trying to change size of a unmanaged window".into())
    }
}
