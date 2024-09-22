use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::state::{NoFallbackBehavior, WManagerLayoutInfo};

use crate::{
    error_handler::Result,
    modules::virtual_desk::get_vd_manager,
    state::application::FULL_STATE,
    windows_api::{monitor::Monitor, window::Window, MonitorEnumerator, WindowsApi},
};

use super::node_impl::WmNodeImpl;

lazy_static! {
    pub static ref WM_STATE: Arc<Mutex<WMV2State>> = Arc::new(Mutex::new({
        let mut state = WMV2State::default();
        state
            .init()
            .expect("Failed to initialize Window Manager State");
        state
    }));
}

#[derive(Debug)]
pub struct WMV2StateWorkspace {
    root: Option<WmNodeImpl>,
    layout_info: Option<WManagerLayoutInfo>,
    no_fallback_behavior: NoFallbackBehavior,
}

#[derive(Debug, Default)]
pub struct WMV2StateMonitor {
    pub id: String,
    pub workspaces: HashMap<String, WMV2StateWorkspace>,
}

#[derive(Debug, Default)]
pub struct WMV2State {
    pub monitors: HashMap<String, WMV2StateMonitor>,
}

impl WMV2State {
    /// will enumarate all monitors and workspaces
    pub fn init(&mut self) -> Result<()> {
        let workspaces = get_vd_manager().get_all()?;
        for (monitor_idx, hmonitor) in MonitorEnumerator::get_all()?.into_iter().enumerate() {
            let id = WindowsApi::monitor_name(hmonitor)?;
            if self.monitors.contains_key(&id) {
                continue;
            }

            let mut monitor = WMV2StateMonitor::default();
            for (workspace_idx, w) in workspaces.iter().enumerate() {
                if monitor.workspaces.contains_key(&w.id()) {
                    continue;
                }

                monitor
                    .workspaces
                    .insert(w.id(), WMV2StateWorkspace::new(monitor_idx, workspace_idx));
            }

            monitor.id = id.clone();
            self.monitors.insert(id, monitor);
        }
        Ok(())
    }

    pub fn get_monitor_mut(&mut self, monitor_id: &str) -> Option<&mut WMV2StateMonitor> {
        self.monitors.get_mut(monitor_id)
    }

    pub fn contains(&self, window: &Window) -> bool {
        self.monitors.values().any(|m| m.contains(window))
    }
}

impl WMV2StateMonitor {
    pub fn create_workspace(monitor_idx: usize, workspace_id: &str) -> Result<WMV2StateWorkspace> {
        for (workspace_idx, w) in get_vd_manager().get_all()?.iter().enumerate() {
            if w.id() == workspace_id {
                return Ok(WMV2StateWorkspace::new(monitor_idx, workspace_idx));
            }
        }
        Err("Invalid workspace id".into())
    }

    pub fn get_workspace_mut(&mut self, workspace_id: &str) -> &mut WMV2StateWorkspace {
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
}

impl WMV2StateWorkspace {
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

    pub fn get_root_node(&self) -> Option<&WmNodeImpl> {
        self.root.as_ref()
    }

    pub fn get_root_node_mut(&mut self) -> Option<&mut WmNodeImpl> {
        self.root.as_mut()
    }

    pub fn add_window(&mut self, window: &Window) {
        let was_added = match self.get_root_node_mut() {
            Some(node) => node.try_add_window(window).is_ok(),
            None => false,
        };

        if !was_added {
            log::warn!("Current Layout is full, and fallback container was not found");
            // TODO
        }
    }

    pub fn remove_window(&mut self, window: &Window) {
        if let Some(node) = self.get_root_node_mut() {
            node.remove_window(window);
        }
    }

    pub fn contains(&self, window: &Window) -> bool {
        self.get_root_node().map_or(false, |n| n.contains(window))
    }
}
