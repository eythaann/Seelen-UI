use slu_ipc::commands::AllowedReservations;
pub use slu_ipc::commands::{Axis, NodeSiblingSide, Sizing, StepWay, WindowManagerCli, WmCommand};

use seelen_core::state::twm::TwmReservation;

use crate::error::Result;
use crate::state::application::FULL_STATE;
use crate::trace_lock;
use crate::virtual_desktops::SluWorkspacesManager2;
use crate::widgets::window_manager::state_v2::{
    set_rect_to_float_initial_size, TwmState, TwmStateEvent, WM_STATE,
};
use crate::windows_api::monitor::Monitor;
use crate::windows_api::window::Window;
use crate::windows_api::MonitorEnumerator;

fn to_wm_reservation(side: AllowedReservations) -> TwmReservation {
    match side {
        AllowedReservations::Left => TwmReservation::Left,
        AllowedReservations::Right => TwmReservation::Right,
        AllowedReservations::Top => TwmReservation::Top,
        AllowedReservations::Bottom => TwmReservation::Bottom,
        AllowedReservations::Stack => TwmReservation::Stack,
        AllowedReservations::Float => TwmReservation::Float,
    }
}

pub fn process(cmd: WindowManagerCli) -> Result<()> {
    process_wm_command(cmd.subcommand)
}

fn process_wm_command(cmd: WmCommand) -> Result<()> {
    let foreground = Window::get_foregrounded();
    let is_moving = matches!(&cmd, WmCommand::Move { .. });

    match cmd {
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
                for instance in &guard.widgets_per_display {
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
            WM_STATE
                .lock()
                .update_size(&foreground, Axis::Horizontal, percentage, false)?;
        }
        WmCommand::Height { action } => {
            let percentage = match action {
                Sizing::Increase => FULL_STATE.load().settings.by_widget.wm.resize_delta,
                Sizing::Decrease => -FULL_STATE.load().settings.by_widget.wm.resize_delta,
            };
            WM_STATE
                .lock()
                .update_size(&foreground, Axis::Vertical, percentage, false)?;
        }
        WmCommand::Reserve { side } => {
            WM_STATE.lock().reserve(to_wm_reservation(side));
        }
        WmCommand::CancelReservation => {
            WM_STATE.lock().cancel_reservation();
        }
        WmCommand::ResetWorkspaceSize => {
            let window_id = foreground.address();
            let mut guard = WM_STATE.lock();
            if guard.state.contains(&window_id) {
                if guard.state.is_tiled(&window_id) {
                    if let Some((_, tree)) = guard.get_tree_for_window_mut(&foreground) {
                        tree.reset_sizes();
                        TwmState::send(TwmStateEvent::Changed);
                    }
                } else {
                    set_rect_to_float_initial_size(&foreground, &foreground.monitor())?;
                }
            }
        }
        WmCommand::ToggleFloat => {
            let mut state = WM_STATE.lock();
            if !state.is_managed(&foreground) {
                return Ok(());
            }
            let workspace_id = foreground.workspace_id()?;
            if state.is_tiled(&foreground) {
                state.remove(&foreground);
                state.add_to_floating(&foreground, &workspace_id);
                set_rect_to_float_initial_size(&foreground, &foreground.monitor())?;
            } else {
                state.remove(&foreground);
                state.add_to_layout(&foreground, &workspace_id);
            }
            TwmState::send(TwmStateEvent::Changed);
        }
        WmCommand::ToggleMonocle => {
            let monitor_id = foreground.monitor_id();
            let workspace_id = SluWorkspacesManager2::instance()
                .monitors
                .get(&monitor_id, |m| m.active_workspace_id().clone())
                .ok_or("Monitor not found")?;
            WM_STATE.lock().toggle_monocle(&workspace_id);
        }
        WmCommand::Focus { side } | WmCommand::Move { side } => {
            let window_id = foreground.address();

            let mut guard = WM_STATE.lock();
            let Some((_ws_id, tree)) = guard.get_tree_for_window_mut(&foreground) else {
                return Ok(());
            };

            let (match_h, want_before) = side_to_flags(&side);
            let siblings = tree.siblings_at_side(&window_id, match_h, want_before);

            let Some(direct_sibling) = siblings.first().and_then(|&nid| tree.face_of_node(nid))
            else {
                log::warn!("There is no direct node at {side:?}");
                drop(guard);
                if is_moving {
                    process_move_to_monitor(&foreground, side)?;
                } else {
                    process_focus_to_monitor(&foreground, side)?;
                }
                return Ok(());
            };

            if is_moving {
                tree.swap_nodes_by_windows(window_id, direct_sibling);
                TwmState::send(TwmStateEvent::Changed);
            } else {
                Window::from(direct_sibling).focus()?;
            }
        }
        WmCommand::MoveToMonitor { side } => {
            process_move_to_monitor(&foreground, side)?;
        }
        WmCommand::CycleStack { way } => {
            WM_STATE.lock().cycle_stack(&foreground, way)?;
        }
    };

    Ok(())
}

fn process_focus_to_monitor(foreground: &Window, side: NodeSiblingSide) -> Result<()> {
    let source_monitor = foreground.monitor();

    let Some(target_monitor) = get_neartest_monitor_at_side(&source_monitor, side)? else {
        log::warn!("There is no monitor at {side:?}");
        return Ok(());
    };

    let Some(target_workspace_id) = SluWorkspacesManager2::instance()
        .monitors
        .get(&target_monitor.stable_id()?, |m| {
            m.active_workspace_id().clone()
        })
    else {
        return Ok(());
    };

    let fg_rect = foreground.inner_rect()?;
    let guard = WM_STATE.lock();
    if let Some(target_window_id) =
        guard.get_nearest_tiled_window_to_rect(&fg_rect, &target_workspace_id)
    {
        Window::from(target_window_id).focus()?;
    }
    Ok(())
}

fn process_move_to_monitor(foreground: &Window, side: NodeSiblingSide) -> Result<()> {
    let source_monitor = foreground.monitor();

    let Some(target_monitor) = get_neartest_monitor_at_side(&source_monitor, side)? else {
        log::warn!("There is no monitor at {side:?}");
        return Ok(());
    };

    if let Some(target_workspace_id) = SluWorkspacesManager2::instance()
        .monitors
        .get(&target_monitor.stable_id()?, |m| {
            m.active_workspace_id().clone()
        })
    {
        let mut guard = WM_STATE.lock();

        guard.remove(foreground);
        guard.add_to_layout(foreground, &target_workspace_id);
        TwmState::send(TwmStateEvent::Changed);
    }
    Ok(())
}

fn side_to_flags(side: &NodeSiblingSide) -> (bool, bool) {
    match side {
        NodeSiblingSide::Left => (true, true),
        NodeSiblingSide::Right => (true, false),
        NodeSiblingSide::Up => (false, true),
        NodeSiblingSide::Down => (false, false),
    }
}

pub fn get_neartest_monitor_at_side(
    monitor: &Monitor,
    side: NodeSiblingSide,
) -> Result<Option<Monitor>> {
    let monitors = MonitorEnumerator::enumerate_win32()?;
    let center = monitor.rect()?.center();

    let mut best: Option<(Monitor, i32)> = None;

    for current in monitors {
        if &current == monitor {
            continue;
        }

        let current_center = current.rect()?.center();

        match side {
            NodeSiblingSide::Left => {
                if current_center.x > center.x {
                    continue;
                }
            }
            NodeSiblingSide::Right => {
                if current_center.x < center.x {
                    continue;
                }
            }
            NodeSiblingSide::Up => {
                if current_center.y > center.y {
                    continue;
                }
            }
            NodeSiblingSide::Down => {
                if current_center.y < center.y {
                    continue;
                }
            }
        }

        let distance = current_center.distance_squared(&center);

        if best.is_none() || distance < best.unwrap().1 {
            best = Some((current, distance));
        }
    }

    Ok(best.map(|(m, _)| m))
}
