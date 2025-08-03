use slu_ipc::messages::{IpcResponse, SvcAction};

use crate::{error::Result, task_scheduler::TaskSchedulerHelper, windows_api::WindowsApi};

async fn _process_action(command: SvcAction) -> Result<()> {
    match command {
        SvcAction::Stop => crate::exit(0),
        SvcAction::SetStartup(enabled) => TaskSchedulerHelper::set_run_on_logon(enabled)?,
        SvcAction::ShowWindow { hwnd, command } => WindowsApi::show_window(hwnd, command)?,
        SvcAction::ShowWindowAsync { hwnd, command } => {
            WindowsApi::show_window_async(hwnd, command)?
        }
        SvcAction::SetWindowPosition {
            hwnd,
            x,
            y,
            width,
            height,
            flags,
        } => WindowsApi::set_position(hwnd, x, y, width, height, flags)?,
        SvcAction::SetForeground(hwnd) => WindowsApi::set_foreground(hwnd)?,
        SvcAction::SetShortcutsConfig(config) => {
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
