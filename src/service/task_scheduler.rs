use windows::Win32::{
    Foundation::{VARIANT_FALSE, VARIANT_TRUE},
    System::TaskScheduler::{
        IExecAction2, ITaskService, TaskScheduler, TASK_ACTION_EXEC, TASK_CREATE_OR_UPDATE,
        TASK_LOGON_INTERACTIVE_TOKEN, TASK_RUNLEVEL_HIGHEST, TASK_TRIGGER_LOGON,
    },
};
use windows_core::{Interface, BSTR};

use crate::{error::Result, windows_api::com::Com};

pub struct TaskSchedulerHelper {}

static GROUP_FOLDER: &str = "\\Seelen";
static OLD_APP_TASK_NAME: &str = "Seelen-UI";
static SERVICE_TASK_NAME: &str = "Seelen UI Service";

impl TaskSchedulerHelper {
    /// this task handles the startup of the service and the app on login
    pub fn create_service_task() -> Result<()> {
        let path = std::env::current_exe()?.to_string_lossy().to_string();
        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(None, None, None, None)?;

            // remove old task as backwards compatibility
            if let Ok(seelen_folder) = task_service.GetFolder(&GROUP_FOLDER.into()) {
                let _ = seelen_folder.DeleteTask(&OLD_APP_TASK_NAME.into(), 0);
            };
            let root_folder = task_service.GetFolder(&"\\".into())?;

            let task = task_service.NewTask(0)?;
            task.Principal()?.SetRunLevel(TASK_RUNLEVEL_HIGHEST)?;

            let settings = task.Settings()?;
            settings.SetPriority(4)?;
            settings.SetHidden(VARIANT_TRUE)?;
            settings.SetAllowDemandStart(VARIANT_TRUE)?;
            settings.SetDisallowStartIfOnBatteries(VARIANT_FALSE)?;
            settings.SetStopIfGoingOnBatteries(VARIANT_FALSE)?;

            let triggers = task.Triggers()?;
            triggers.Create(TASK_TRIGGER_LOGON)?;

            let actions = task.Actions()?;
            let exec_action: IExecAction2 = actions.Create(TASK_ACTION_EXEC)?.cast()?;
            exec_action.SetPath(&path.into())?;

            let mut task_xml = BSTR::new();
            task.XmlText(&mut task_xml)?;
            root_folder.RegisterTask(
                &format!("{GROUP_FOLDER}\\{SERVICE_TASK_NAME}").into(),
                &task_xml,
                TASK_CREATE_OR_UPDATE.0,
                None,
                None,
                TASK_LOGON_INTERACTIVE_TOKEN,
                None,
            )?;
            Ok(())
        })
    }

    pub fn set_enabled_service_task(enabled: bool) -> Result<()> {
        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(None, None, None, None)?;
            let seelen_folder = task_service.GetFolder(&GROUP_FOLDER.into())?;
            let task = seelen_folder.GetTask(&SERVICE_TASK_NAME.into())?;
            task.SetEnabled(if enabled { VARIANT_TRUE } else { VARIANT_FALSE })?;
            Ok(())
        })
    }

    pub fn remove_service_task() -> Result<()> {
        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(None, None, None, None)?;
            if let Ok(seelen_folder) = task_service.GetFolder(&GROUP_FOLDER.into()) {
                let _ = seelen_folder.DeleteTask(&SERVICE_TASK_NAME.into(), 0);
            }
            Ok(())
        })
    }
}
