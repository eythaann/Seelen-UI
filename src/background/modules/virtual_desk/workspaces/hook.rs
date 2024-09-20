use crate::{
    error_handler::Result,
    log_error,
    modules::virtual_desk::get_vd_manager,
    seelen_weg::SeelenWeg,
    windows_api::{window::Window, WindowEnumerator},
    winevent::WinEvent,
};

use super::SeelenWorkspacesManager;

impl SeelenWorkspacesManager {
    fn should_be_added(window: &Window) -> bool {
        SeelenWeg::should_be_added(window.hwnd()) && !window.is_minimized()
    }

    fn contains(&self, window: &Window) -> bool {
        self.with_workspaces(|workspaces| {
            workspaces
                .iter()
                .any(|w| w.windows.contains(&window.address()))
        })
    }

    fn add(&self, window: &Window) {
        self.with_workspace(self.current_idx(), |workspace| {
            log::trace!("adding window to workspace: {}", window.address());
            workspace.windows.push(window.address());
        });
    }

    /// TODO: try to move windows on others native virtual desktops to only one.
    pub fn enumerate(&self) -> Result<()> {
        WindowEnumerator::new()
            .map(|hwnd| hwnd)?
            .into_iter()
            .rev()
            .for_each(|hwnd| {
                let window = Window::from(hwnd);
                if Self::should_be_added(&window) {
                    self.add(&window);
                }
            });
        Ok(())
    }

    pub fn on_win_event(&self, event: WinEvent, window: &Window) -> Result<()> {
        let addr = window.address();
        match event {
            WinEvent::ObjectCreate | WinEvent::ObjectShow => {
                if Self::should_be_added(window) && !self.contains(window) {
                    self.add(window);
                }
            }
            WinEvent::SystemMinimizeEnd => {
                let owner_idx = self.with_workspaces(|workspaces| {
                    workspaces.iter().position(|w| w.windows.contains(&addr))
                });

                if let Some(owner_idx) = owner_idx {
                    std::thread::spawn(move || log_error!(get_vd_manager().switch_to(owner_idx)));
                } else if !self.contains(window) && Self::should_be_added(window) {
                    self.add(window);
                }
            }
            WinEvent::ObjectDestroy | WinEvent::ObjectHide | WinEvent::SystemMinimizeStart => {
                self.with_workspace(self.current_idx(), |workspace| {
                    if workspace.windows.contains(&addr) {
                        log::trace!("removing window: {}", addr);
                        workspace.windows.retain(|w| *w != addr);
                    }
                });
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                self.with_workspaces(|workspaces| {
                    for w in workspaces {
                        if w.windows.contains(&addr) {
                            w.windows.retain(|w| *w != addr);
                            w.windows.push(addr);
                            break;
                        }
                    }
                });
            }
            WinEvent::ObjectParentChange => {
                if let Some(parent) = window.parent() {
                    if self.contains(window) {
                        self.with_workspaces(|workspaces| {
                            workspaces
                                .iter_mut()
                                .for_each(|w| w.windows.retain(|w| *w != addr));
                        })
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
