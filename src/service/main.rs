// #![windows_subsystem = "windows"]

mod app;
mod cli;
mod error;
mod logger;
mod string_utils;
mod task_scheduler;
mod windows_api;

use cli::{handle_cli, ServiceClient};
use lazy_static::lazy_static;
use logger::SluServiceLogger;
use parking_lot::Mutex;
use std::sync::Arc;
use string_utils::WindowsString;
use task_scheduler::TaskSchedulerHelper;
use windows::Win32::{
    Foundation::{GetLastError, FALSE, HANDLE, NO_ERROR, TRUE, WIN32_ERROR},
    Security::{
        AllocateAndInitializeSid,
        Authorization::{
            ConvertSidToStringSidW, SetEntriesInAclW, EXPLICIT_ACCESS_W, SET_ACCESS,
            TRUSTEE_IS_SID, TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_W,
        },
        GetSecurityDescriptorDacl, InitializeSecurityDescriptor, SetSecurityDescriptorDacl, ACL,
        DACL_SECURITY_INFORMATION, NO_INHERITANCE, PSECURITY_DESCRIPTOR, PSID, SECURITY_DESCRIPTOR,
        SECURITY_WORLD_SID_AUTHORITY, SE_TCB_NAME,
    },
    System::{
        Services::{
            ChangeServiceConfig2W, CreateServiceW, DeleteService, OpenSCManagerW, OpenServiceW,
            QueryServiceObjectSecurity, RegisterServiceCtrlHandlerExW, SetServiceObjectSecurity,
            SetServiceStatus, StartServiceCtrlDispatcherW, SC_HANDLE, SC_MANAGER_ALL_ACCESS,
            SERVICE_ACCEPT_STOP, SERVICE_ALL_ACCESS, SERVICE_CONFIG_DESCRIPTION,
            SERVICE_CONTROL_STOP, SERVICE_DEMAND_START, SERVICE_DESCRIPTIONW, SERVICE_ERROR_NORMAL,
            SERVICE_RUNNING, SERVICE_START, SERVICE_START_PENDING, SERVICE_STATUS,
            SERVICE_STATUS_CURRENT_STATE, SERVICE_STATUS_HANDLE, SERVICE_STOP, SERVICE_STOPPED,
            SERVICE_STOP_PENDING, SERVICE_TABLE_ENTRYW, SERVICE_WIN32_OWN_PROCESS,
        },
        SystemServices::{SECURITY_DESCRIPTOR_REVISION, SECURITY_WORLD_RID},
        Threading::{CreateEventW, SetEvent, WaitForSingleObject, INFINITE},
    },
};
use windows_api::WindowsApi;
use windows_core::PWSTR;

use crate::error::Result;

lazy_static! {
    pub static ref SERVICE_NAME: WindowsString = WindowsString::from_str("slu-service");
    pub static ref SERVICE_DISPLAY_NAME: WindowsString =
        WindowsString::from_str("Seelen UI Service");
    static ref SLU_SERVICE: Arc<Mutex<SluService>> = Arc::new(Mutex::new(SluService::new()));
}

/// All documentation is taken from https://learn.microsoft.com/en-us/windows/win32/services/services
struct SluService {
    handle: SERVICE_STATUS_HANDLE,
    status: SERVICE_STATUS,
    stop_event: HANDLE,
}

unsafe impl Send for SluService {}

impl SluService {
    unsafe fn _update_service_security(sc_handle: SC_HANDLE) -> Result<()> {
        let mut bytes_needed = 0;
        let mut security_descriptor = SECURITY_DESCRIPTOR::default();
        let psecurity_descriptor = PSECURITY_DESCRIPTOR(&mut security_descriptor as *mut _ as _);

        // first call to get the needed buffer size
        let _ = QueryServiceObjectSecurity(
            sc_handle,
            DACL_SECURITY_INFORMATION.0,
            psecurity_descriptor,
            0,
            &mut bytes_needed,
        );

        QueryServiceObjectSecurity(
            sc_handle,
            DACL_SECURITY_INFORMATION.0,
            psecurity_descriptor,
            bytes_needed,
            &mut bytes_needed,
        )?;

        let mut acl_ptr = std::ptr::null_mut::<ACL>();
        let mut dacl_present = FALSE;
        let mut dacl_default = FALSE;
        GetSecurityDescriptorDacl(
            psecurity_descriptor,
            &mut dacl_present,
            &mut acl_ptr,
            &mut dacl_default,
        )?;

        // println!("ACL present: {}", dacl_present.as_bool());
        // println!("ACL default: {}", dacl_default.as_bool());
        // println!("ACL: {:#?}", acl_ptr.as_ref());

        let mut everyone_psid = PSID::default();
        AllocateAndInitializeSid(
            &SECURITY_WORLD_SID_AUTHORITY,
            1,
            SECURITY_WORLD_RID as u32,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut everyone_psid,
        )?;

        let mut everyone_psid_str = PWSTR::null();
        ConvertSidToStringSidW(everyone_psid, &mut everyone_psid_str)?;

        let explitic_access = [EXPLICIT_ACCESS_W {
            grfAccessPermissions: SERVICE_START | SERVICE_STOP,
            grfAccessMode: SET_ACCESS,
            grfInheritance: NO_INHERITANCE,
            Trustee: TRUSTEE_W {
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_WELL_KNOWN_GROUP,
                ptstrName: everyone_psid_str,
                ..Default::default()
            },
        }];

        let mut new_acl_ptr = std::ptr::null_mut::<ACL>();
        SetEntriesInAclW(
            Some(&explitic_access),
            if acl_ptr.is_null() {
                None
            } else {
                Some(&*acl_ptr)
            },
            &mut new_acl_ptr,
        )
        .ok()?;
        let new_acl = new_acl_ptr.as_ref().ok_or("new_acl_ptr is null")?;

        // Initialize a new security descriptor.
        let new_security_descriptor = PSECURITY_DESCRIPTOR::default();
        InitializeSecurityDescriptor(new_security_descriptor, SECURITY_DESCRIPTOR_REVISION)?;
        // let new_security_descriptor = new_security_descriptor.0 as *mut SECURITY_DESCRIPTOR;

        // Set the new DACL in the security descriptor.
        SetSecurityDescriptorDacl(new_security_descriptor, TRUE, Some(new_acl), FALSE)?;

        // Set the new DACL for the service object.
        SetServiceObjectSecurity(
            sc_handle,
            DACL_SECURITY_INFORMATION,
            new_security_descriptor,
        )?;
        Ok(())
    }

    fn is_running() -> bool {
        ServiceClient::connect_tcp().is_ok()
    }

    fn install() -> Result<()> {
        let service_path = std::env::current_exe()?;
        let sc_manager_handle = unsafe { OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS)? };
        let exists = unsafe {
            OpenServiceW(
                sc_manager_handle,
                SERVICE_NAME.as_pcwstr(),
                SERVICE_ALL_ACCESS,
            )
        }
        .is_ok();

        if exists {
            println!("Service was already installed");
            return Ok(());
        }

        unsafe {
            let sc_handle = CreateServiceW(
                sc_manager_handle,
                SERVICE_NAME.as_pcwstr(),
                SERVICE_DISPLAY_NAME.as_pcwstr(),
                SERVICE_ALL_ACCESS,
                SERVICE_WIN32_OWN_PROCESS,
                SERVICE_DEMAND_START,
                SERVICE_ERROR_NORMAL,
                WindowsString::from_str(format!("\"{}\"", service_path.display())).as_pcwstr(),
                None,
                None,
                None,
                None, // LocalSystem account
                None,
            )?;

            let mut description_str =
                WindowsString::from_str("Relaunch Seelen UI on crash and handle system actions");
            let description = SERVICE_DESCRIPTIONW {
                lpDescription: description_str.as_pwstr(),
            };

            ChangeServiceConfig2W(
                sc_handle,
                SERVICE_CONFIG_DESCRIPTION,
                Some(&description as *const _ as *const _),
            )?;
            // this is always failing and I don't find a way to fix it so a workaround was implemented
            // https://stackoverflow.com/questions/39960007/reasons-for-setentriesinacl-error-87-in-msdn-sample
            // Self::_update_service_security(sc_handle)?;
        }
        Ok(())
    }

    fn uninstall() -> Result<()> {
        let sc_manager_handle = unsafe { OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS)? };
        let sc_handle = unsafe {
            OpenServiceW(
                sc_manager_handle,
                SERVICE_NAME.as_pcwstr(),
                SERVICE_ALL_ACCESS,
            )
            .ok()
        };
        if let Some(sc_handle) = sc_handle {
            unsafe { DeleteService(sc_handle)? };
        }
        Ok(())
    }

    fn new() -> Self {
        Self {
            handle: SERVICE_STATUS_HANDLE::default(),
            status: SERVICE_STATUS {
                dwServiceType: SERVICE_WIN32_OWN_PROCESS,
                ..SERVICE_STATUS::default()
            },
            stop_event: HANDLE::default(),
        }
    }

    fn set_status(
        &mut self,
        state: SERVICE_STATUS_CURRENT_STATE,
        exit_code: Option<WIN32_ERROR>,
        wait_hint: Option<u32>,
    ) {
        self.status.dwCurrentState = state;
        self.status.dwWin32ExitCode = exit_code.unwrap_or(NO_ERROR).0;
        self.status.dwWaitHint = wait_hint.unwrap_or(0);

        if state != SERVICE_START_PENDING {
            self.status.dwControlsAccepted = SERVICE_ACCEPT_STOP;
        }

        if state == SERVICE_RUNNING || state == SERVICE_STOPPED {
            self.status.dwCheckPoint = 0;
        } else {
            self.status.dwCheckPoint += 1;
        }

        let result = unsafe { SetServiceStatus(self.handle, &self.status) };
        match result {
            Ok(_) => {
                log::trace!("Service status updated to: {:?}", self.status);
            }
            Err(err) => {
                log::error!("Failed to set service status: {}", err);
            }
        }
    }

    fn init(&mut self, handle: SERVICE_STATUS_HANDLE) -> Result<HANDLE> {
        self.handle = handle;
        self.set_status(SERVICE_START_PENDING, None, Some(3000));

        self.stop_event = unsafe {
            CreateEventW(
                None,  // default security attributes
                TRUE,  // manual reset event
                FALSE, // not signaled
                None,
            )?
        };

        WindowsApi::enable_privilege(SE_TCB_NAME)?;
        ServiceClient::listen_tcp()?;
        TaskSchedulerHelper::run_app_startup_task()?;

        self.set_status(SERVICE_RUNNING, None, None);
        Ok(self.stop_event)
    }

    fn stop(&mut self) {
        self.set_status(SERVICE_STOP_PENDING, None, None);
        unsafe { SetEvent(self.stop_event) }.expect("Failed to emit stop event");
    }
}

unsafe extern "system" fn service_control_handler(
    dwcontrol: u32,
    _dweventtype: u32,
    _lpeventdata: *mut core::ffi::c_void,
    _lpcontext: *mut core::ffi::c_void,
) -> u32 {
    #[allow(clippy::single_match)]
    match dwcontrol {
        SERVICE_CONTROL_STOP => {
            SLU_SERVICE.lock().stop();
        }
        _ => {}
    }
    0
}

unsafe extern "system" fn service_main(
    _dwnumservicesargs: u32,
    _lpserviceargvectors: *mut windows_core::PWSTR,
) {
    SluServiceLogger::init().expect("Failed to initialize logger");
    log::info!("Starting Seelen UI Service");

    let sc_status_handle = RegisterServiceCtrlHandlerExW(
        WindowsString::from_str("slu-service").as_pcwstr(),
        Some(service_control_handler),
        None,
    )
    .expect("Failed to register service control handler");

    let stop_event = {
        let mut service = SLU_SERVICE.lock();
        match service.init(sc_status_handle) {
            Ok(event) => event,
            Err(e) => {
                log::error!("Failed to initialize service: {}", e);
                service.set_status(SERVICE_STOPPED, Some(GetLastError()), None);
                return;
            }
        }
    };

    // this will lock until the event is signalled (the event should be manually emitted)
    WaitForSingleObject(stop_event, INFINITE);
    SLU_SERVICE.lock().set_status(SERVICE_STOPPED, None, None);
    log::info!("Seelen UI Service stopped");
}

fn main() -> Result<()> {
    handle_cli()?;
    let mut service_name = SERVICE_NAME.clone();
    let table = SERVICE_TABLE_ENTRYW {
        lpServiceName: service_name.as_pwstr(),
        lpServiceProc: Some(service_main),
    };
    // This call locks until the service is stopped
    // Will return error if isn't executing the program as service (ex: invoking the executable directly)
    unsafe { StartServiceCtrlDispatcherW(&table) }?;
    Ok(())
}
