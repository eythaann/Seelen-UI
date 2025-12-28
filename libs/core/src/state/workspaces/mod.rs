use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{error::Result, identifier_impl, resource::WallpaperId, system_state::MonitorId};

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
    pub workspaces: Vec<DesktopWorkspace>,
    active_workspace: WorkspaceId,
}

impl VirtualDesktopMonitor {
    pub fn create() -> Self {
        let workspace = DesktopWorkspace::create();
        let current_workspace = workspace.id.clone();
        Self {
            workspaces: vec![workspace],
            active_workspace: current_workspace,
        }
    }

    pub fn sanitize(&mut self) {
        if self.workspaces.is_empty() {
            let workspace = DesktopWorkspace::create();
            self.active_workspace = workspace.id.clone();
            self.workspaces.push(workspace);
        }

        if !self
            .workspaces
            .iter()
            .any(|ws| ws.id == self.active_workspace)
        {
            self.active_workspace = self.workspaces[0].id.clone();
        }
    }

    pub fn active_workspace_id(&self) -> &WorkspaceId {
        &self.active_workspace
    }

    pub fn active_workspace(&self) -> &DesktopWorkspace {
        self.workspaces
            .iter()
            .find(|w| w.id == self.active_workspace)
            .expect("current workspace not found")
    }

    pub fn active_workspace_mut(&mut self) -> &mut DesktopWorkspace {
        self.workspaces
            .iter_mut()
            .find(|w| w.id == self.active_workspace)
            .expect("current workspace not found")
    }

    /// Set the current workspace, return error if the workspace doesn't exist
    pub fn set_active_workspace(&mut self, workspace_id: &WorkspaceId) -> Result<()> {
        if self.workspaces.iter().any(|w| &w.id == workspace_id) {
            self.active_workspace = workspace_id.clone();
            Ok(())
        } else {
            Err("Invalid workspace id".into())
        }
    }

    /// Add a new workspace and return its id
    pub fn add_workspace(&mut self) -> WorkspaceId {
        let workspace = DesktopWorkspace::create();
        let workspace_id = workspace.id.clone();
        self.workspaces.push(workspace);
        workspace_id
    }

    /// Remove a workspace by id
    /// - Does nothing if there's only 1 workspace (minimum required)
    /// - If the removed workspace was current, switches to the side one
    /// - Moves all windows from the removed workspace to the side one in the array
    pub fn remove_workspace(&mut self, workspace_id: &WorkspaceId) -> Result<()> {
        // Don't remove if it's the only workspace
        if self.workspaces.len() <= 1 {
            return Err("Cannot remove the last workspace".into());
        }

        // Find the index of the workspace to remove
        let idx_to_delete = self
            .workspaces
            .iter()
            .position(|w| &w.id == workspace_id)
            .ok_or("Workspace not found")?;

        let idx_to_move = if idx_to_delete == 0 {
            1
        } else {
            idx_to_delete - 1
        };

        // If the removed workspace was current, switch to the previous one
        if &self.active_workspace == workspace_id {
            self.active_workspace = self.workspaces[idx_to_move].id.clone();
        }

        // Move windows to the side workspace
        let windows = self.workspaces[idx_to_delete].windows.clone();
        for window in windows {
            self.workspaces[idx_to_move].windows.push(window);
        }
        self.workspaces.remove(idx_to_delete);
        Ok(())
    }

    /// Rename a workspace by id
    pub fn rename_workspace(
        &mut self,
        workspace_id: &WorkspaceId,
        name: Option<String>,
    ) -> Result<()> {
        let workspace = self
            .workspaces
            .iter_mut()
            .find(|w| &w.id == workspace_id)
            .ok_or("Workspace not found")?;
        workspace.name = name;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct DesktopWorkspace {
    pub id: WorkspaceId,
    pub name: Option<String>,
    /// react-icon icon name
    pub icon: Option<String>,
    pub wallpaper: Option<WallpaperId>,
    #[serde(default)]
    pub windows: Vec<isize>,
}

impl DesktopWorkspace {
    pub fn create() -> Self {
        Self {
            id: WorkspaceId(Uuid::new_v4().to_string()),
            name: None,
            icon: None,
            wallpaper: None,
            windows: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, TS)]
pub struct WorkspaceId(pub String);

identifier_impl!(WorkspaceId, String);
