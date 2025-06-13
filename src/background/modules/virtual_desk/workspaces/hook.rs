use crate::{
    error_handler::Result,
    log_error,
    modules::virtual_desk::get_vd_manager,
    trace_lock,
    windows_api::{window::Window, WindowEnumerator},
    winevent::WinEvent,
};

use super::SeelenWorkspacesManager;

impl SeelenWorkspacesManager {
    fn should_be_added(window: &Window) -> bool {
        window.is_real_window() && !window.is_minimized()
    }

    fn contains(&self, window: &Window) -> bool {
        trace_lock!(self.workspaces)
            .iter()
            .any(|w| w.windows.contains(&window.address()))
    }

    fn add(&self, window: &Window) {
        if let Some(workspace) = trace_lock!(self.workspaces).get_mut(self.current_idx()) {
            log::trace!("adding window to workspace: {}", window.address());
            workspace.windows.push(window.address());
        };
    }

    /// TODO: try to move windows on others native virtual desktops to only one.
    pub fn enumerate(&self) -> Result<()> {
        WindowEnumerator::new().for_each(|window| {
            if Self::should_be_added(&window) {
                self.add(&window);
            }
        })
    }

    pub fn on_win_event(&self, event: WinEvent, window: &Window) -> Result<()> {
        let addr = window.address();
        match event {
            WinEvent::ObjectCreate | WinEvent::ObjectShow | WinEvent::ObjectNameChange => {
                if !self.contains(window) && Self::should_be_added(window) {
                    self.add(window);
                }
            }
            WinEvent::SystemMinimizeEnd => {
                let owner_idx = trace_lock!(self.workspaces)
                    .iter()
                    .position(|w| w.windows.contains(&addr));

                if let Some(owner_idx) = owner_idx {
                    std::thread::spawn(move || log_error!(get_vd_manager().switch_to(owner_idx)));
                } else if !self.contains(window) && Self::should_be_added(window) {
                    self.add(window);
                }
            }
            WinEvent::ObjectDestroy | WinEvent::ObjectHide | WinEvent::SystemMinimizeStart => {
                if let Some(workspace) = trace_lock!(self.workspaces).get_mut(self.current_idx()) {
                    if workspace.windows.contains(&addr) {
                        log::trace!("removing window: {addr}");
                        workspace.windows.retain(|w| *w != addr);
                    }
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                for w in trace_lock!(self.workspaces).iter_mut() {
                    if w.windows.contains(&addr) {
                        w.windows.retain(|w| *w != addr);
                        w.windows.push(addr);
                        break;
                    }
                }
            }
            WinEvent::ObjectParentChange => {
                if let Some(parent) = window.parent() {
                    if self.contains(window) {
                        trace_lock!(self.workspaces)
                            .iter_mut()
                            .for_each(|w| w.windows.retain(|w| *w != addr));
                    }

                    if !self.contains(&parent) && Self::should_be_added(&parent) {
                        self.add(&parent);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
