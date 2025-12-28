use std::sync::LazyLock;

use positioning::{easings::Easing, AnimationOrchestrator, PositionerBuilder};
use slu_ipc::messages::{IpcResponse, SvcAction};

use crate::{error::Result, task_scheduler::TaskSchedulerHelper, windows_api::WindowsApi};

static ANIMATION_ORCHESTRATOR: LazyLock<AnimationOrchestrator> =
    LazyLock::new(AnimationOrchestrator::new);

async fn _process_action(command: SvcAction) -> Result<()> {
    match command {
        SvcAction::Stop => crate::exit(0),
        // -----------------------------------------------------------------------
        SvcAction::SetStartup(enabled) => TaskSchedulerHelper::set_run_on_logon(enabled)?,
        SvcAction::ShowWindow { hwnd, command } => WindowsApi::show_window(hwnd, command)?,
        SvcAction::ShowWindowAsync { hwnd, command } => {
            WindowsApi::show_window_async(hwnd, command)?
        }
        SvcAction::SetWindowPosition { hwnd, rect, flags } => WindowsApi::set_position(
            hwnd,
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            flags,
        )?,
        SvcAction::DeferWindowPositions {
            list,
            animated,
            animation_duration,
            easing,
        } => {
            let mut builder = PositionerBuilder::new();
            for (hwnd, rect) in list {
                builder.add(
                    hwnd,
                    positioning::rect::Rect {
                        x: rect.left,
                        y: rect.top,
                        width: rect.right - rect.left,
                        height: rect.bottom - rect.top,
                    },
                );
            }

            if !animated {
                builder.place()?;
                return Ok(());
            }

            let easing = Easing::from_name(&easing).unwrap_or(Easing::Linear);
            // The orchestrator will automatically interrupt only the windows in this batch
            // if they're already animating, without affecting other windows
            ANIMATION_ORCHESTRATOR.animate_batch(
                builder.build(),
                animation_duration,
                easing,
                move |result| {
                    if let Err(err) = result {
                        log::error!("Animated window placement failed: {err}");
                    }
                },
            )?;
        }
        SvcAction::SetForeground(hwnd) => WindowsApi::set_foreground(hwnd)?,
        SvcAction::SetSettings(settings) => {
            if settings.shortcuts.enabled {
                crate::hotkeys::start_app_shortcuts(&settings)?;
            } else {
                crate::hotkeys::stop_app_shortcuts();
            }
        }
        SvcAction::StartShortcutRegistration => {
            crate::hotkeys::start_shortcut_registration().await?;
        }
        SvcAction::StopShortcutRegistration => {
            crate::hotkeys::stop_shortcut_registration().await?;
        }
    }
    Ok(())
}

pub async fn process_action(command: SvcAction) -> IpcResponse {
    log::trace!("Processing action: {:?}", command);
    match _process_action(command).await {
        Ok(()) => IpcResponse::Success,
        Err(err) => IpcResponse::Err(err.to_string()),
    }
}
