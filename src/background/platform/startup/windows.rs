use std::path::Path;

use windows::Win32::System::TaskScheduler::{IExecAction2, ITaskService, TaskScheduler};
use windows_core::{BSTR, Interface};

use crate::{error::Result, windows_api::Com};

use super::StartupApi;

const TASK_FOLDER: &str = r"\Seelen";
const TASK_NAME: &str = "Seelen UI Service";

pub struct WindowsStartup;

impl StartupApi for WindowsStartup {
    fn is_enabled() -> Result<bool> {
        Com::run_with_context(|| unsafe {
            let task_service = connect_task_service()?;
            // Task Scheduler collections are 1-based
            let is_task_enabled = task_service
                .GetFolder(&TASK_FOLDER.into())
                .and_then(|folder| folder.GetTask(&TASK_NAME.into()))
                .and_then(|task| task.Definition())
                .and_then(|definition| definition.Triggers())
                .and_then(|triggers| triggers.get_Item(1))
                .is_ok();
            Ok(is_task_enabled)
        })
    }

    fn launch_service(service_exe: &Path) -> Result<()> {
        Com::run_with_context(|| unsafe {
            let task_service = connect_task_service()?;
            let folder = task_service.GetFolder(&TASK_FOLDER.into())?;
            let task = folder.GetTask(&TASK_NAME.into())?;

            // Task Scheduler collections are 1-based
            let action: IExecAction2 = task.Definition()?.Actions()?.get_Item(1)?.cast()?;
            let mut action_path = BSTR::new();
            action.Path(&mut action_path)?;

            let expected = service_exe.to_string_lossy().to_lowercase();
            if action_path.to_string().to_lowercase() != expected {
                return Err(
                    "Service task is not pointing to the correct service executable".into(),
                );
            }

            task.Run(&Default::default())?;
            Ok(())
        })
    }
}

/// Must be called inside a COM context.
unsafe fn connect_task_service() -> Result<ITaskService> {
    unsafe {
        let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
        task_service.Connect(
            &Default::default(),
            &Default::default(),
            &Default::default(),
            &Default::default(),
        )?;
        Ok(task_service)
    }
}
