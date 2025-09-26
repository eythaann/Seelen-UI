use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::state::application::FULL_STATE;
use crate::trace_lock;
use crate::virtual_desktops::get_vd_manager;
use crate::widgets::window_manager::node_ext::WmNodeExt;
use crate::widgets::window_manager::state::WmWorkspaceState;
use crate::windows_api::window::Window;

use super::instance::WindowManagerV2;
use super::state::WM_STATE;

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum AllowedReservations {
    Left,
    Right,
    Top,
    Bottom,
    Stack,
    Float,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum NodeSiblingSide {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Sizing {
    Increase,
    Decrease,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum StepWay {
    Next,
    Prev,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Axis {
    Horizontal,
    Vertical,
    Top,
    Bottom,
    Left,
    Right,
}

/// Manage the Seelen Window Manager.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
#[command(alias = "wm")]
pub struct WindowManagerCli {
    #[command(subcommand)]
    pub subcommand: WmCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum WmCommand {
    /// Open Dev Tools (only works if the app is running in dev mode)
    Debug,
    /// Toggles the Seelen Window Manager.
    Toggle,
    /// Reserve space for a incoming window.
    Reserve {
        /// The position of the new window.
        side: AllowedReservations,
    },
    /// Cancels the current reservation
    CancelReservation,
    /// Increases or decreases the size of the window
    Width {
        /// What to do with the width.
        action: Sizing,
    },
    /// Increases or decreases the size of the window
    Height {
        /// What to do with the height.
        action: Sizing,
    },
    /// Resets the size of the containers in current workspace to the default size.
    ResetWorkspaceSize,
    /// Toggles the floating state of the window
    ToggleFloat,
    /// Toggles workspace layout mode to monocle (single stack)
    ToggleMonocle,
    /// Moves the window to the specified position
    Move { side: NodeSiblingSide },
    /// Cycles the foregrounf node if it is a stack
    CycleStack { way: StepWay },
    /// Focuses the window in the specified position.
    Focus {
        /// The position of the window to focus.
        side: NodeSiblingSide,
    },
}

impl WindowManagerCli {
    pub fn process(self) -> Result<()> {
        self.subcommand.process()
    }
}

impl WmCommand {
    pub fn process(self) -> Result<()> {
        let foreground = Window::get_foregrounded();

        match self {
            WmCommand::Toggle => {
                FULL_STATE.rcu(move |state| {
                    let mut state = state.cloned();
                    state.settings.by_widget.wm.enabled = !state.settings.by_widget.wm.enabled;
                    state
                });
                FULL_STATE.load().write_settings()?;
            }
            WmCommand::Debug => {
                #[cfg(debug_assertions)]
                {
                    let guard = trace_lock!(crate::app::SEELEN);
                    for instance in &guard.instances {
                        if let Some(wm) = &instance.wm {
                            wm.window.open_devtools();
                        }
                    }
                }
            }
            WmCommand::Width { action } => {
                let percentage = match action {
                    Sizing::Increase => FULL_STATE.load().settings.by_widget.wm.resize_delta,
                    Sizing::Decrease => -FULL_STATE.load().settings.by_widget.wm.resize_delta,
                };

                let mut state = trace_lock!(WM_STATE);
                state.update_size(&foreground, Axis::Horizontal, percentage, false)?;

                let monitor_id = foreground.monitor_id();
                let current_workspace = get_vd_manager()
                    .get_active_workspace_id(&monitor_id)
                    .clone();
                WindowManagerV2::render_workspace(
                    &monitor_id,
                    state.get_workspace_state(&current_workspace),
                )?;
            }
            WmCommand::Height { action } => {
                let percentage = match action {
                    Sizing::Increase => FULL_STATE.load().settings.by_widget.wm.resize_delta,
                    Sizing::Decrease => -FULL_STATE.load().settings.by_widget.wm.resize_delta,
                };

                let mut state = trace_lock!(WM_STATE);
                state.update_size(&foreground, Axis::Vertical, percentage, false)?;

                let monitor_id = foreground.monitor_id();
                let current_workspace = get_vd_manager()
                    .get_active_workspace_id(&monitor_id)
                    .clone();
                WindowManagerV2::render_workspace(
                    &monitor_id,
                    state.get_workspace_state(&current_workspace),
                )?;
            }
            WmCommand::Reserve { .. } => {
                // self.reserve(side)?;
            }
            WmCommand::CancelReservation => {
                // self.discard_reservation()?;
            }
            WmCommand::ResetWorkspaceSize => {
                let mut state = trace_lock!(WM_STATE);
                if let Some(workspace) = state.get_workspace_state_for_window(&foreground) {
                    if workspace.is_floating(&foreground.address()) {
                        WmWorkspaceState::set_rect_to_float_initial_size(&foreground)?;
                    }
                }
            }
            WmCommand::ToggleFloat => {
                let mut state = trace_lock!(WM_STATE);
                if let Some(workspace) = state.get_workspace_state_for_window(&foreground) {
                    if workspace.is_floating(&foreground.address()) {
                        workspace.add_to_tiles(&foreground);
                    } else if workspace.is_tiled(&foreground) {
                        workspace.unmanage(&foreground);
                        workspace.add_to_floats(&foreground)?;
                    }

                    WindowManagerV2::render_workspace(&foreground.monitor_id(), workspace)?;
                }
            }
            WmCommand::ToggleMonocle => {
                let monitor_id = foreground.monitor_id();
                let workspace = get_vd_manager()
                    .get_active_workspace_id(&monitor_id)
                    .clone();

                let mut state = trace_lock!(WM_STATE);
                let workspace = state.get_workspace_state(&workspace);
                workspace.toggle_monocle();
                WindowManagerV2::render_workspace(&monitor_id, workspace)?;
            }
            WmCommand::Focus { side } => {
                let mut state = trace_lock!(WM_STATE);
                if let Some(workspace) = state.get_workspace_state_for_window(&foreground) {
                    let siblings = workspace
                        .layout
                        .structure
                        .get_siblings_at_side(&foreground, &side);
                    match siblings.first().and_then(|sibling| sibling.face()) {
                        Some(sibling) => {
                            sibling.focus()?;
                        }
                        None => {
                            log::warn!("There is no node at {side:?} to be focused");
                        }
                    }
                }
            }
            WmCommand::Move { side } => {
                let mut state = trace_lock!(WM_STATE);
                if let Some(workspace) = state.get_workspace_state_for_window(&foreground) {
                    let siblings = workspace
                        .layout
                        .structure
                        .get_siblings_at_side(&foreground, &side);

                    match siblings.first().and_then(|sibling| sibling.face()) {
                        Some(sibling) => {
                            workspace.swap_nodes_containing_window(&foreground, &sibling)?;
                            WindowManagerV2::render_workspace(&foreground.monitor_id(), workspace)?;
                        }
                        None => {
                            log::warn!("There is no node at {side:?} to be swapped");
                        }
                    }
                }
            }
            WmCommand::CycleStack { way } => {
                let mut state = trace_lock!(WM_STATE);
                let Some(workspace) = state.get_workspace_state_for_window(&foreground) else {
                    return Ok(());
                };
                let Some(node) = workspace.layout.structure.leaf_containing_mut(&foreground) else {
                    return Ok(());
                };

                let active = node.active.ok_or("No active window")?;
                let idx = node
                    .windows
                    .iter()
                    .position(|w| *w == active)
                    .ok_or("No active window")?;

                let len = node.windows.len();
                let next_idx = if way == StepWay::Next {
                    (idx + 1) % len // next and cycle
                } else {
                    (idx + (len - 1)) % len // prev and cycle
                };

                node.active = Some(node.windows[next_idx]);

                WindowManagerV2::render_workspace(&foreground.monitor_id(), workspace)?;
            }
        };

        Ok(())
    }
}
