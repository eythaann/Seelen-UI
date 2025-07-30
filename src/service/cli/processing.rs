use slu_ipc::messages::{SvcAction, SvcResponse};

use crate::{error::Result, task_scheduler::TaskSchedulerHelper, windows_api::WindowsApi};

fn _process_action(command: SvcAction) -> Result<()> {
    match command {
        SvcAction::Stop => crate::stop(),
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
    }
    Ok(())
}

pub fn process_action(command: SvcAction) -> SvcResponse {
    match _process_action(command) {
        Ok(()) => SvcResponse::Success,
        Err(err) => SvcResponse::Err(err.to_string()),
    }
}
