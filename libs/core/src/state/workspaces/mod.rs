use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{identifier_impl, system_state::MonitorId};

#[derive(Debug, Default, Clone, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct VirtualDesktops {
    /// Workspaces per monitor
    pub monitors: HashMap<MonitorId, VirtualDesktopMonitor>,
    /// pinned windows will be not affected by switching workspaces
    pub pinned: Vec<isize>,
}

impl VirtualDesktops {
    pub fn sanitize(&mut self) {
        let mut seen = HashSet::new();
        self.pinned.retain(|x| seen.insert(*x));

        for monitor in self.monitors.values_mut() {
            monitor.sanitize();
            for workspace in &mut monitor.workspaces {
                workspace.windows.retain(|x| seen.insert(*x));
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct VirtualDesktopMonitor {
    pub id: MonitorId,
    pub workspaces: Vec<DesktopWorkspace>,
    pub current_workspace: WorkspaceId,
}

impl VirtualDesktopMonitor {
    pub fn create(id: MonitorId) -> Self {
        let workspace = DesktopWorkspace::create();
        let current_workspace = workspace.id.clone();
        Self {
            id,
            workspaces: vec![workspace],
            current_workspace,
        }
    }

    pub fn sanitize(&mut self) {
        if self.workspaces.is_empty() {
            let workspace = DesktopWorkspace::create();
            self.current_workspace = workspace.id.clone();
            self.workspaces.push(workspace);
        }

        if !self
            .workspaces
            .iter()
            .any(|ws| ws.id == self.current_workspace)
        {
            self.current_workspace = self.workspaces[0].id.clone();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DesktopWorkspace {
    pub id: WorkspaceId,
    pub name: Option<String>,
    /// react-icon icon name
    pub icon: Option<String>,
    #[serde(default)]
    pub windows: Vec<isize>,
}

impl DesktopWorkspace {
    pub fn create() -> Self {
        Self {
            id: WorkspaceId(Uuid::new_v4().to_string()),
            name: None,
            icon: None,
            windows: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
pub struct WorkspaceId(pub String);

identifier_impl!(WorkspaceId, String);
