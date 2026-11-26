use sysinfo::ProcessesToUpdate;
use windows::Win32::System::RemoteDesktop::ProcessIdToSessionId;

use crate::{
    error::Result, modules::app_management::domain::AppManagementSystem, windows_api::WindowsApi,
};

const APP_EXE_NAME: &str = "seelen-ui.exe";
const APP_MSIX_AUMID: &str = "Seelen.SeelenUI_p6yyn03m1894e!App";

pub struct WindowsAppManagement;

impl AppManagementSystem for WindowsAppManagement {
    fn create() -> Self {
        Self
    }

    fn launch_app(&self) -> Result<()> {
        let path = if WindowsApi::was_installed_using_msix() {
            format!("shell:AppsFolder\\{APP_MSIX_AUMID}")
        } else {
            std::env::current_exe()?
                .with_file_name(APP_EXE_NAME)
                .to_string_lossy()
                .to_string()
        };
        // Use explorer.exe to spawn the app de-elevated (with the interactive user's token),
        // since the service runs elevated (TASK_RUNLEVEL_HIGHEST). ShellExecuteExW called
        // from an elevated process inherits the elevated token, causing an infinite restart loop.
        std::process::Command::new("explorer.exe")
            .arg(&path)
            .spawn()?;
        Ok(())
    }

    fn kill_app_instances(&self) -> Result<()> {
        let current_session = WindowsApi::current_session_id();

        let mut sys = sysinfo::System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let instances = sys.processes().values().filter(|p| {
            if !p.exe().is_some_and(|path| path.ends_with(APP_EXE_NAME)) {
                return false;
            }
            let mut session_id = 0u32;
            let in_session =
                unsafe { ProcessIdToSessionId(p.pid().as_u32(), &mut session_id).is_ok() };
            in_session && session_id == current_session
        });
        for instance in instances {
            instance.kill();
        }
        Ok(())
    }
}
