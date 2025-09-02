use crate::{
    error::Result,
    trace_lock,
    virtual_desktops::{events::VirtualDesktopEvent, SluWorkspacesManager, HIDDEN_BY_USER},
    windows_api::{
        window::{event::WinEvent, Window},
        WindowEnumerator,
    },
};

impl SluWorkspacesManager {
    pub fn should_be_added(window: &Window) -> bool {
        window.is_real_window()
    }

    pub fn contains(&self, window: &Window) -> bool {
        let window_id = window.address();
        self.is_pinned(&window_id)
            || self
                .iter_workspaces()
                .any(|w| w.windows.contains(&window_id))
    }

    fn add(&mut self, window: &Window) {
        let window_id = window.address();
        let Ok(monitor_id) = window.monitor().stable_id() else {
            // as fallback we gonna add the window to the pinned list.
            // if getting monitor id continues to fail, this won't be able to be unpinned.
            self.0.pinned.push(window_id);
            return;
        };

        let active = self.get_active_workspace_mut(&monitor_id.into());
        if active.windows.contains(&window_id) {
            return;
        }

        log::trace!("adding window ({window_id:x}) to workspace {}", active.id);
        active.windows.push(window_id);

        Self::send(VirtualDesktopEvent::WindowAdded {
            window: window_id,
            desktop: active.id.clone(),
        });
        self.save()
    }

    fn remove(&mut self, window: &Window) {
        let window_id = window.address();
        self.0.pinned.retain(|w| w != &window_id);
        for workspace in self.iter_workspaces_mut() {
            workspace.windows.retain(|w| w != &window_id);
        }
        Self::send(VirtualDesktopEvent::WindowRemoved { window: window_id });
        self.save()
    }

    /// TODO: try to move windows on others native virtual desktops to only one,
    /// or add a warning message to users.
    pub fn list_windows_into_respective_workspace(&mut self) -> Result<()> {
        // restore workspaces state
        for (monitor_id, monitor) in &self.desktops().monitors {
            for workspace in &monitor.workspaces {
                if workspace.id == monitor.current_workspace {
                    Self::restore_workspace(workspace);
                    Self::event_tx().send(VirtualDesktopEvent::DesktopChanged {
                        monitor: monitor_id.clone(),
                        workspace: workspace.id.clone(),
                    })?;
                } else {
                    Self::hide_workspace(workspace, false);
                }
            }
        }

        // scan new windows, but only add the visible ones to the active workspace
        WindowEnumerator::new().for_each(|window| {
            if !self.contains(&window) && Self::should_be_added(&window) && !window.is_minimized() {
                self.add(&window);
            }
        })
    }

    pub fn on_win_event(&mut self, event: WinEvent, window: &Window) -> Result<()> {
        let window_id = window.address();
        match event {
            WinEvent::ObjectCreate | WinEvent::ObjectShow => {
                if !self.contains(window) && Self::should_be_added(window) {
                    self.add(window);
                }
            }
            // ObjectNameChange, is fired on styles changed too, so we need to revaluate them,
            // also there's filters by title, so this will be revaluated too.
            WinEvent::ObjectNameChange => match self.contains(window) {
                true => {
                    if !Self::should_be_added(window) {
                        self.remove(window);
                    }
                }
                false => {
                    if Self::should_be_added(window) {
                        self.add(window);
                    }
                }
            },
            WinEvent::SystemMinimizeStart => {
                trace_lock!(HIDDEN_BY_USER).insert(window_id);
            }
            WinEvent::SystemMinimizeEnd => {
                trace_lock!(HIDDEN_BY_USER).remove(&window_id);
                if let Some(workspace) =
                    self.workspace_containing_window(&window.address()).cloned()
                {
                    self.switch_to_id(&window.monitor().stable_id()?.into(), &workspace.id)?;
                } else if !self.is_pinned(&window_id) && Self::should_be_added(window) {
                    // add minimized windows during the scanning, to the current active workspace
                    self.add(window);
                }
            }
            WinEvent::ObjectDestroy | WinEvent::ObjectHide => {
                self.remove(window);
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                if let Some(workspace) = self.workspace_containing_window_mut(&window_id) {
                    // update z-order
                    workspace.windows.retain(|w| w != &window_id);
                    workspace.windows.push(window_id);
                    self.save();
                }
            }
            WinEvent::ObjectParentChange => {
                if let Some(parent) = window.parent() {
                    if self.contains(window) {
                        self.remove(window);
                    }

                    if !self.contains(&parent) && Self::should_be_added(&parent) {
                        self.add(&parent);
                    }
                }
            }
            WinEvent::SyntheticMonitorChanged => {
                if self.contains(window) && !self.is_pinned(&window_id) {
                    self.remove(window);
                    self.add(window);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
