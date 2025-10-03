use std::sync::LazyLock;

use positioning::{easings::Easing, AppWinAnimation, Positioner};
use seelen_core::state::shortcuts::SluShortcutsSettings;
use slu_ipc::messages::{IpcResponse, SvcAction};

use crate::{error::Result, task_scheduler::TaskSchedulerHelper, windows_api::WindowsApi};

static ANIMATION_INSTANCE: LazyLock<tokio::sync::Mutex<Option<AppWinAnimation>>> =
    LazyLock::new(|| tokio::sync::Mutex::new(None));

async fn _process_action(command: SvcAction) -> Result<()> {
    match command {
        SvcAction::Stop => crate::exit(0),
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
            // the guards avoid playing multiple animations at the same time.
            let mut guard = ANIMATION_INSTANCE.lock().await;
            if let Some(mut last) = guard.take() {
                last.interrupt();
                last.wait();
            }

            let mut positioner = Positioner::new();
            for (hwnd, rect) in list {
                positioner.add(
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
                positioner.place()?;
                return Ok(());
            }

            let easing = Easing::from_name(&easing).unwrap_or(Easing::Linear);
            let animation =
                positioner.place_animated(animation_duration, easing, move |result| {
                    if let Err(err) = result {
                        log::error!("Animated window placement failed: {err}");
                    }
                })?;
            *guard = Some(animation);
        }
        SvcAction::SetForeground(hwnd) => WindowsApi::set_foreground(hwnd)?,
        SvcAction::SetShortcutsConfig(config) => {
            let config: SluShortcutsSettings = serde_json::from_str(&config)?;
            if config.enabled {
                crate::hotkeys::start_app_shortcuts(config)?;
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
    match _process_action(command).await {
        Ok(()) => IpcResponse::Success,
        Err(err) => IpcResponse::Err(err.to_string()),
    }
}
