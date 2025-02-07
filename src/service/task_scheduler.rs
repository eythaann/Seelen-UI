use windows::Win32::{
    Foundation::{VARIANT_FALSE, VARIANT_TRUE},
    System::TaskScheduler::{
        IExecAction2, ITaskFolder, ITaskService, TaskScheduler, TASK_ACTION_EXEC,
        TASK_CREATE_OR_UPDATE, TASK_LOGON_INTERACTIVE_TOKEN, TASK_RUNLEVEL_HIGHEST,
        TASK_TRIGGER_LOGON,
    },
    UI::Shell::FOLDERID_LocalAppData,
};
use windows_core::{Interface, BSTR};

use crate::{
    enviroment::was_installed_using_msix,
    error::Result,
    windows_api::{com::Com, WindowsApi},
};

pub struct TaskSchedulerHelper {}

static GROUP_FOLDER: &str = "\\Seelen";
static OLD_APP_TASK_NAME: &str = "Seelen-UI";
static SERVICE_TASK_NAME: &str = "Seelen UI Service";

impl TaskSchedulerHelper {
    unsafe fn get_task_service() -> Result<ITaskService> {
        let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
        task_service.Connect(None, None, None, None)?;
        Ok(task_service)
    }

    unsafe fn register_task(folder: &ITaskFolder, task_name: &str, task_xml: &BSTR) -> Result<()> {
        folder.RegisterTask(
            &task_name.into(),
            task_xml,
            TASK_CREATE_OR_UPDATE.0,
            None,
            None,
            TASK_LOGON_INTERACTIVE_TOKEN,
            None,
        )?;
        Ok(())
    }

    /// this task handles the startup of the service and the app on login
    pub fn create_service_task() -> Result<()> {
        let service_path = if was_installed_using_msix() {
            WindowsApi::known_folder(FOLDERID_LocalAppData)?
                .join("Microsoft\\WindowsApps\\slu-service.exe")
        } else {
            std::env::current_exe()?
        };
        Com::run_with_context(|| unsafe {
            let task_service = Self::get_task_service()?;
            // remove old task as backwards compatibility
            let mut old_task = None;
            if let Ok(seelen_folder) = task_service.GetFolder(&GROUP_FOLDER.into()) {
                let _ = seelen_folder.DeleteTask(&OLD_APP_TASK_NAME.into(), 0);
                old_task = seelen_folder.GetTask(&OLD_APP_TASK_NAME.into()).ok();
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
            if let Some(old) = old_task {
                let old_triggers = old.Definition()?.Triggers()?;
                task.SetTriggers(&old_triggers)?;
            } else {
                triggers.Create(TASK_TRIGGER_LOGON)?;
            }

            let actions = task.Actions()?;
            let exec_action: IExecAction2 = actions.Create(TASK_ACTION_EXEC)?.cast()?;
            exec_action.SetPath(&service_path.to_string_lossy().to_string().into())?;

            let mut task_xml = BSTR::new();
            task.XmlText(&mut task_xml)?;
            Self::register_task(
                &root_folder,
                &format!("{GROUP_FOLDER}\\{SERVICE_TASK_NAME}"),
                &task_xml,
            )?;
            Ok(())
        })
    }

    pub fn set_run_on_logon(enabled: bool) -> Result<()> {
        Com::run_with_context(|| unsafe {
            let task_service = Self::get_task_service()?;
            let seelen_folder = task_service.GetFolder(&GROUP_FOLDER.into())?;
            let task = seelen_folder.GetTask(&SERVICE_TASK_NAME.into())?;
            let task = task.Definition()?;
            let triggers = task.Triggers()?;
            triggers.Clear()?;
            if enabled {
                triggers.Create(TASK_TRIGGER_LOGON)?;
            }
            let mut task_xml = BSTR::new();
            task.XmlText(&mut task_xml)?;
            Self::register_task(&seelen_folder, SERVICE_TASK_NAME, &task_xml)?;
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
