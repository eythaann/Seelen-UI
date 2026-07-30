use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{error::Result, identifier_impl, resource::WallpaperId, system_state::MonitorId};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct VirtualDesktops {
    /// Workspaces per monitor
    pub monitors: HashMap<MonitorId, VirtualDesktopMonitor>,
    /// pinned windows will be not affected by switching workspaces
    pub pinned: Vec<isize>,
    /// true if the system is currently switching workspaces
    pub switching: bool,
}

impl VirtualDesktops {
    pub fn sanitize(&mut self) {
        let mut seen = HashSet::new();
        self.pinned.retain(|x| seen.insert(*x));

        for monitor in self.monitors.values_mut() {
            monitor.sanitize();
            /* for workspace in &mut monitor.workspaces {
                workspace.windows.retain(|x| seen.insert(*x));
            } */
            for row in &mut monitor.workspaces.0 {
                for workspace in row {
                    workspace.windows.retain(|x| seen.insert(*x));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct VirtualDesktopMonitor {
    pub workspaces: WorkspaceGrid,
    active_workspace: WorkspaceId,
}

impl VirtualDesktopMonitor {
    pub fn create() -> Self {
        let workspace = DesktopWorkspace::create();
        let current_workspace = workspace.id.clone();
        Self {
            workspaces: WorkspaceGrid(vec![vec![workspace]]),
            active_workspace: current_workspace,
        }
    }

    pub fn sanitize(&mut self) {
        self.workspaces.sanitize();

        if !self.workspaces.contains(&self.active_workspace) {
            self.active_workspace = self.workspaces.0[0][0].id.clone();
        }
    }

    pub fn active_workspace_id(&self) -> &WorkspaceId {
        &self.active_workspace
    }

    pub fn active_row(&self) -> &Vec<DesktopWorkspace> {
        for row in &self.workspaces.0 {
            if row.iter().any(|x| x.id == self.active_workspace) {
                return row;
            }
        }
        panic!("Active workspace doesn't exist in the grid");
    }

    pub fn active_workspace(&self) -> &DesktopWorkspace {
        self.workspaces
            .get_by_id(&self.active_workspace)
            .expect("current workspace not found")
    }

    pub fn active_workspace_mut(&mut self) -> &mut DesktopWorkspace {
        self.workspaces
            .get_by_id_mut(&self.active_workspace)
            .expect("current workspace not found")
    }

    /// Set the current workspace, return error if the workspace doesn't exist
    pub fn set_active_workspace(&mut self, workspace_id: &WorkspaceId) -> Result<()> {
        if self.workspaces.contains(workspace_id) {
            self.active_workspace = workspace_id.clone();
            Ok(())
        } else {
            Err("Invalid workspace id".into())
        }
    }

    /// Add a new workspace and return its id
    pub fn add_workspace_column(&mut self) -> WorkspaceId {
        self.workspaces.add_column();
        let row = self.active_row();
        let workspace_id = row.last().unwrap().id.clone();
        self.active_workspace = workspace_id.clone();
        workspace_id
    }

    /// Remove a workspace by id
    /// - Removing a workspace means removing its whole column across every row (minimum 1 column required)
    /// - If the removed workspace was current, switches to the side one
    /// - Moves all windows from each removed cell to the side column in its own row
    pub fn remove_workspace(&mut self, workspace_id: &WorkspaceId) -> Result<()> {
        // Don't remove if it's the last column
        if self.workspaces.column_count() <= 1 {
            return Err("Cannot remove the last workspace".into());
        }

        // Find the column to remove
        let (_, col_idx) = self
            .workspaces
            .position(workspace_id)
            .ok_or("Workspace not found")?;

        let col_to_move = if col_idx == 0 { 1 } else { col_idx - 1 };

        let mut new_active = None;
        for row in &mut self.workspaces.0 {
            let windows = row[col_idx].windows.clone();
            row[col_to_move].windows.extend(windows);

            if row[col_idx].id == self.active_workspace {
                new_active = Some(row[col_to_move].id.clone());
            }

            row.remove(col_idx);
        }

        if let Some(active) = new_active {
            self.active_workspace = active;
        }

        self.workspaces.sanitize();
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
            .get_by_id_mut(workspace_id)
            .ok_or("Workspace not found")?;
        workspace.name = name;
        Ok(())
    }

    pub fn set_wallpapers(&mut self, f: impl Fn(&DesktopWorkspace) -> Option<WallpaperId>) {
        for row in &mut self.workspaces.0 {
            for workspace in row {
                workspace.wallpaper = f(workspace);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct WorkspaceGrid(Vec<Vec<DesktopWorkspace>>);

impl WorkspaceGrid {
    pub fn rows(&self) -> &Vec<Vec<DesktopWorkspace>> {
        &self.0
    }

    pub fn rows_mut(&mut self) -> &mut Vec<Vec<DesktopWorkspace>> {
        &mut self.0
    }

    pub fn row_count(&self) -> usize {
        self.0.len()
    }

    pub fn column_count(&self) -> usize {
        self.0[0].len()
    }

    /// Ensures the grid has at least one row/column (1x1 minimum) and that
    /// every row has the same number of columns, padding short rows with new workspaces.
    pub fn sanitize(&mut self) {
        self.0.retain(|row| !row.is_empty());

        if self.0.is_empty() {
            self.0.push(vec![DesktopWorkspace::create()]);
            return;
        }

        let max_columns = self.0.iter().map(|row| row.len()).max().unwrap_or(1).max(1);
        for row in &mut self.0 {
            while row.len() < max_columns {
                row.push(DesktopWorkspace::create());
            }
        }
    }

    /// Find the (row, column) position of a workspace by id
    pub fn position(&self, workspace_id: &WorkspaceId) -> Option<(usize, usize)> {
        for (row_idx, row) in self.0.iter().enumerate() {
            for (col_idx, workspace) in row.iter().enumerate() {
                if &workspace.id == workspace_id {
                    return Some((row_idx, col_idx));
                }
            }
        }
        None
    }

    pub fn add_row(&mut self) {
        let mut row = Vec::new();
        for _ in 0..self.column_count() {
            row.push(DesktopWorkspace::create());
        }
        self.0.push(row);
    }

    pub fn add_column(&mut self) {
        for row in &mut self.0 {
            row.push(DesktopWorkspace::create());
        }
    }

    pub fn contains(&self, workspace_id: &WorkspaceId) -> bool {
        for row in &self.0 {
            for workspace in row {
                if &workspace.id == workspace_id {
                    return true;
                }
            }
        }
        false
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&DesktopWorkspace> {
        self.0.get(row)?.get(col)
    }

    pub fn get_by_window_id(&self, window_id: isize) -> Option<&DesktopWorkspace> {
        for row in &self.0 {
            for workspace in row {
                if workspace.windows.contains(&window_id) {
                    return Some(workspace);
                }
            }
        }
        None
    }

    pub fn get_by_window_id_mut(&mut self, window_id: isize) -> Option<&mut DesktopWorkspace> {
        for row in &mut self.0 {
            for workspace in row {
                if workspace.windows.contains(&window_id) {
                    return Some(workspace);
                }
            }
        }
        None
    }

    pub fn get_by_id(&self, workspace_id: &WorkspaceId) -> Option<&DesktopWorkspace> {
        for row in &self.0 {
            for workspace in row {
                if &workspace.id == workspace_id {
                    return Some(workspace);
                }
            }
        }
        None
    }

    pub fn get_by_id_mut(&mut self, workspace_id: &WorkspaceId) -> Option<&mut DesktopWorkspace> {
        for row in &mut self.0 {
            for workspace in row {
                if &workspace.id == workspace_id {
                    return Some(workspace);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct WorkspaceId(pub String);

identifier_impl!(WorkspaceId, String);
