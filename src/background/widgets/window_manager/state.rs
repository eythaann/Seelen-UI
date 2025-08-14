use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::{
    state::{
        value::{KnownPlugin, PluginValue},
        NoFallbackBehavior, WmNode, WorkspaceId,
    },
    system_state::MonitorId,
};

use crate::{
    error_handler::Result,
    modules::{input::domain::Point, monitors::MonitorManager},
    state::application::FULL_STATE,
    virtual_desktops::get_vd_manager,
    windows_api::{monitor::Monitor, window::Window},
};

use super::{cli::Axis, node_impl::WmNodeImpl};

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
    no_fallback_behavior: NoFallbackBehavior,
}

#[derive(Debug)]
pub struct WmV2StateMonitor {
    pub id: MonitorId,
    pub workspaces: HashMap<WorkspaceId, WmV2StateWorkspace>,
}

#[derive(Debug, Default)]
pub struct WmV2State {
    pub monitors: HashMap<MonitorId, WmV2StateMonitor>,
}

#[allow(dead_code)]
impl WmV2State {
    /// will enumarate all monitors and workspaces
    pub fn init(&mut self) -> Result<()> {
        let mut guard = get_vd_manager();

        for view in MonitorManager::get_all_views()? {
            let monitor_id = view.primary_target()?.stable_id2()?;
            if self.monitors.contains_key(&monitor_id) {
                continue;
            }

            let mut wm_monitor = WmV2StateMonitor::new(&monitor_id);

            let vd_monitor = guard.get_monitor_mut(&monitor_id);
            for workspace in &vd_monitor.workspaces {
                if wm_monitor.workspaces.contains_key(&workspace.id) {
                    continue;
                }

                wm_monitor.workspaces.insert(
                    workspace.id.clone(),
                    WmV2StateWorkspace::new(&monitor_id, &workspace.id),
                );
            }

            wm_monitor.id = monitor_id.clone();
            self.monitors.insert(monitor_id, wm_monitor);
        }
        Ok(())
    }

    pub fn get_monitor_mut(&mut self, monitor_id: &MonitorId) -> Option<&mut WmV2StateMonitor> {
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
        if let Some(s_monitor) = self.monitors.get_mut(&monitor.stable_id2().ok()?) {
            let current_workspace = get_vd_manager()
                .get_monitor_mut(&s_monitor.id)
                .current_workspace
                .clone();

            if let Some(w) = s_monitor.workspaces.get_mut(&current_workspace) {
                return w.get_node_at_point(point);
            }
        }
        None
    }

    /// # Parameters
    ///
    /// - `window`: A reference to the window whose size is being updated.
    /// - `axis`: The axis along which the size update will occur (horizontal or vertical).
    /// - `percentage`: The percentage by which the window size will be updated. Can be positive or negative.
    /// - `relative`: Determines how the percentage is interpreted. If `true`, the percentage is relative to
    ///   the current window size. If `false`, it's relative to the total workspace size.
    ///
    pub fn update_size(
        &self,
        window: &Window,
        axis: Axis,
        percentage: f32,
        relative: bool,
    ) -> Result<(&WmV2StateMonitor, &WmV2StateWorkspace)> {
        if let Some((m, w, trace)) = self.trace_to(window) {
            let mut siblins_with_window_node = &Vec::new();

            for n in trace.iter().rev() {
                match n {
                    WmNode::Horizontal(inner) => match axis {
                        Axis::Horizontal | Axis::Left | Axis::Right => {
                            if inner
                                .children
                                .iter()
                                .filter(|n| !n.is_empty())
                                .collect_vec()
                                .len()
                                >= 2
                            {
                                siblins_with_window_node = &inner.children;
                                break;
                            }
                        }
                        _ => {}
                    },
                    WmNode::Vertical(inner) => match axis {
                        Axis::Horizontal | Axis::Top | Axis::Bottom => {
                            if inner
                                .children
                                .iter()
                                .filter(|n| !n.is_empty())
                                .collect_vec()
                                .len()
                                >= 2
                            {
                                siblins_with_window_node = &inner.children;
                                break;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            if siblins_with_window_node.is_empty() {
                log::warn!("Can't change size if the window is alone on axis");
                return Ok((m, w));
            }

            let (node_of_window_idx, node_of_window) = siblins_with_window_node
                .iter()
                .find_position(|n| WmNodeImpl::new((*n).clone()).contains(window))
                .expect("The algorithm at the top of this function is wrong / broken");

            let siblins = siblins_with_window_node
                .iter()
                .enumerate()
                .filter(|(idx, n)| {
                    *idx != node_of_window_idx
                        && match axis {
                            Axis::Horizontal | Axis::Vertical => true,
                            Axis::Left | Axis::Top => *idx < node_of_window_idx,
                            Axis::Bottom | Axis::Right => *idx > node_of_window_idx,
                        }
                        && !n.is_empty()
                })
                .collect_vec();

            if siblins.is_empty() {
                log::warn!(
                    "Can't change size at {axis:?} if there are no other windows on that side"
                );
                return Ok((m, w));
            }

            let total_pie: f32 = siblins.iter().map(|(_, n)| n.grow_factor().get()).sum();
            let window_portion = node_of_window.grow_factor().get();

            let to_grow = if relative { window_portion } else { total_pie } * percentage / 100f32;
            let to_reduce = to_grow / siblins.len() as f32;

            node_of_window.grow_factor().set(window_portion + to_grow);
            for (_, n) in siblins {
                n.grow_factor().set(n.grow_factor().get() - to_reduce);
            }
            return Ok((m, w));
        }

        Err("Trying to change size of an unmanaged window".into())
    }
}

#[allow(dead_code)]
impl WmV2StateMonitor {
    pub fn new(monitor_id: &MonitorId) -> Self {
        Self {
            id: monitor_id.clone(),
            workspaces: HashMap::new(),
        }
    }

    pub fn get_workspace_mut(&mut self, workspace_id: &WorkspaceId) -> &mut WmV2StateWorkspace {
        self.workspaces
            .entry(workspace_id.clone())
            .or_insert_with(|| WmV2StateWorkspace::new(&self.id, workspace_id))
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

#[allow(dead_code)]
impl WmV2StateWorkspace {
    pub fn new(monitor: &MonitorId, workspace: &WorkspaceId) -> Self {
        let settings = FULL_STATE.load();
        let layout_id = settings.get_wm_layout_id(monitor, workspace);

        let mut workspace = Self {
            root: None,
            no_fallback_behavior: NoFallbackBehavior::Float,
        };

        let plugin_with_layout = settings.plugins().values().find(|p| p.id == layout_id);
        let Some(plugin) = plugin_with_layout else {
            return workspace;
        };
        let PluginValue::Known(plugin) = &plugin.plugin else {
            return workspace;
        };
        let KnownPlugin::WManager(layout) = plugin else {
            return workspace;
        };

        workspace.root = Some(WmNodeImpl::new(layout.structure.clone()));
        workspace.no_fallback_behavior = layout.no_fallback_behavior.clone();
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
        self.root.as_ref().is_some_and(|n| n.contains(window))
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
