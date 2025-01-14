use windows::Win32::{
    Foundation::{VARIANT_FALSE, VARIANT_TRUE},
    System::TaskScheduler::{
        IExecAction2, ITaskService, TaskScheduler, TASK_ACTION_EXEC, TASK_CREATE_OR_UPDATE,
        TASK_LOGON_INTERACTIVE_TOKEN, TASK_RUNLEVEL_HIGHEST, TASK_TRIGGER_LOGON,
    },
};
use windows_core::{Interface, BSTR};

use crate::{error::Result, string_utils::WindowsString, windows_api::com::Com};

pub struct TaskSchedulerHelper {}

static GROUP_FOLDER: &str = "\\Seelen";
static OLD_APP_TASK_NAME: &str = "Seelen-UI";
static APP_TASK_NAME: &str = "Seelen UI App";
static SERVICE_TASK_NAME: &str = "Seelen UI Service";

impl TaskSchedulerHelper {
    fn run_task(task_name: &str) -> Result<()> {
        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(None, None, None, None)?;
            let seelen_folder = task_service.GetFolder(&GROUP_FOLDER.into())?;
            // could fail for many reasons ex: task is already running
            // https://learn.microsoft.com/es-es/windows/win32/api/taskschd/nf-taskschd-iregisteredtask-run
            let _ = seelen_folder.GetTask(&task_name.into())?.Run(None);
            Ok(())
        })
    }

    pub fn create_service_task() -> Result<()> {
        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(None, None, None, None)?;
            let seelen_folder = task_service.GetFolder(&GROUP_FOLDER.into())?;

            let task = task_service.NewTask(0)?;
            task.Principal()?.SetRunLevel(TASK_RUNLEVEL_HIGHEST)?;

            let settings = task.Settings()?;
            settings.SetHidden(VARIANT_TRUE)?;
            settings.SetAllowDemandStart(VARIANT_TRUE)?;
            settings.SetDisallowStartIfOnBatteries(VARIANT_FALSE)?;
            settings.SetStopIfGoingOnBatteries(VARIANT_FALSE)?;

            let actions = task.Actions()?;
            let exec_action: IExecAction2 = actions.Create(TASK_ACTION_EXEC)?.cast()?;
            exec_action.SetPath(&"net".into())?;
            exec_action.SetArguments(&"start slu-service".into())?;

            let mut task_xml = BSTR::new();
            task.XmlText(&mut task_xml)?;
            seelen_folder.RegisterTask(
                &SERVICE_TASK_NAME.into(),
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

    pub fn run_service_task() -> Result<()> {
        Self::run_task(SERVICE_TASK_NAME)
    }

    pub fn remove_service_task() -> Result<()> {
        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(None, None, None, None)?;
            let seelen_folder = task_service.GetFolder(&GROUP_FOLDER.into())?;
            let _ = seelen_folder.DeleteTask(&SERVICE_TASK_NAME.into(), 0);
            Ok(())
        })
    }

    pub fn create_app_startup_task() -> Result<()> {
        let app_path = WindowsString::from_os_string(
            std::env::current_exe()?
                .with_file_name("seelen-ui.exe")
                .as_os_str(),
        );

        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(None, None, None, None)?;
            let seelen_folder = task_service.GetFolder(&GROUP_FOLDER.into())?;

            // remove old task as backwards compatibility
            let _ = seelen_folder.DeleteTask(&OLD_APP_TASK_NAME.into(), 0);

            let task = task_service.NewTask(0)?;

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
            exec_action.SetPath(&app_path.to_bstr())?;
            exec_action.SetArguments(&"--silent".into())?;

            let mut task_xml = BSTR::new();
            task.XmlText(&mut task_xml)?;

            seelen_folder.RegisterTask(
                &APP_TASK_NAME.into(),
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

    pub fn run_app_startup_task() -> Result<()> {
        Self::run_task(APP_TASK_NAME)
    }

    pub fn remove_app_startup_task() -> Result<()> {
        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(None, None, None, None)?;
            let seelen_folder = task_service.GetFolder(&GROUP_FOLDER.into())?;
            let _ = seelen_folder.DeleteTask(&OLD_APP_TASK_NAME.into(), 0); // backwards compatibility
            let _ = seelen_folder.DeleteTask(&APP_TASK_NAME.into(), 0);
            Ok(())
        })
    }
}
