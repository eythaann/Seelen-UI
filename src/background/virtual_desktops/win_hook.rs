use crate::{
    error::Result,
    modules::apps::application::{UserAppsEvent, UserAppsManager},
    trace_lock,
    virtual_desktops::{
        events::VirtualDesktopEvent, SluWorkspacesManager, MINIMIZED_BY_USER,
        MINIMIZED_BY_WORKSPACES, WORKSPACES_MANAGER,
    },
    windows_api::window::{event::WinEvent, Window},
};

impl SluWorkspacesManager {
    pub fn should_be_added(window: &Window) -> bool {
        window.is_interactable_and_not_hidden()
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

        log::trace!("adding {window} to workspace {}", active.id);
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

        // scan no added windows, but only add the non minimized ones to the current active workspace
        UserAppsManager::instance()
            .interactable_windows
            .for_each(|data| {
                let window = Window::from(data.hwnd);
                if !self.contains(&window) && !window.is_minimized() {
                    self.add(&window);
                }
            });
        Ok(())
    }

    pub(super) fn init_hook_listener() {
        UserAppsManager::subscribe(|event| match event {
            UserAppsEvent::WinAdded(addr) => {
                let mut guard = trace_lock!(WORKSPACES_MANAGER);
                guard.add(&Window::from(addr));
            }
            UserAppsEvent::WinRemoved(addr) => {
                let mut guard = trace_lock!(WORKSPACES_MANAGER);
                guard.remove(&Window::from(addr));
            }
            _ => {}
        });
    }

    pub fn on_win_event(&mut self, event: WinEvent, window: &Window) -> Result<()> {
        let window_id = window.address();
        match event {
            WinEvent::SystemMinimizeStart => {
                if !MINIMIZED_BY_WORKSPACES.contains(&window_id) {
                    let _ = MINIMIZED_BY_USER.insert(window_id);
                }
            }
            WinEvent::SystemMinimizeEnd => {
                MINIMIZED_BY_USER.remove(&window_id);
                if let Some(workspace) =
                    self.workspace_containing_window(&window.address()).cloned()
                {
                    self.switch_to_id(&window.monitor().stable_id()?.into(), &workspace.id)?;
                } else if !self.is_pinned(&window_id) && Self::should_be_added(window) {
                    // add minimized windows during the scanning, to the current active workspace
                    self.add(window);
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                if let Some(workspace) = self.workspace_containing_window_mut(&window_id) {
                    // update z-order
                    workspace.windows.retain(|w| w != &window_id);
                    workspace.windows.push(window_id);
                    self.save();
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
