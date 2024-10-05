use std::path::PathBuf;

use windows::{
    ApplicationModel::AppInfo,
    Win32::{
        Foundation::HANDLE,
        Storage::Packaging::Appx::{
            GetApplicationUserModelId, GetPackageFamilyName, GetPackageFullName,
        },
        System::Threading::PROCESS_QUERY_INFORMATION,
    },
};

use crate::error_handler::Result;

use super::{string_utils::WindowsString, window::Window, WindowsApi};

// https://stackoverflow.com/questions/47300622/meaning-of-flags-in-process-extended-basic-information-struct
#[allow(dead_code)]
pub enum ProcessInformationFlag {
    IsProtectedProcess = 0x1,
    IsWow64Process = 0x2,
    IsProcessInJob = 0x4,
    IsCrossSessionCreate = 0x8,
    IsFrozen = 0x10,
    IsBackground = 0x20,
    IsStronglyNamed = 0x40,
    IsSecureProcess = 0x80,
    IsSubsystemProcess = 0x100,
}

pub struct Process(u32);

impl Process {
    pub fn from_window(window: &Window) -> Self {
        let (process_id, _) = WindowsApi::window_thread_process_id(window.hwnd());
        Self(process_id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    fn with_handle<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(HANDLE) -> T,
    {
        let handle = WindowsApi::open_process(PROCESS_QUERY_INFORMATION, false, self.0)?;
        let result = f(handle);
        WindowsApi::close_handle(handle)?;
        Ok(result)
    }

    pub fn package_family_name(&self) -> Result<String> {
        self.with_handle(|hprocess| {
            let mut len = 1024_u32;
            let mut family_name = WindowsString::new_to_fill(len as usize);
            unsafe { GetPackageFamilyName(hprocess, &mut len, family_name.as_pwstr()).ok()? };
            Ok(family_name.to_string())
        })?
    }

    pub fn package_full_name(&self) -> Result<String> {
        self.with_handle(|hprocess| {
            let mut len = 1024_u32;
            let mut family_name = WindowsString::new_to_fill(len as usize);
            unsafe { GetPackageFullName(hprocess, &mut len, family_name.as_pwstr()).ok()? };
            Ok(family_name.to_string())
        })?
    }

    /// package app user model id
    pub fn package_app_user_model_id(&self) -> Result<String> {
        self.with_handle(|hprocess| {
            let mut len = 1024_u32;
            let mut id = WindowsString::new_to_fill(len as usize);
            unsafe { GetApplicationUserModelId(hprocess, &mut len, id.as_pwstr()).ok()? };
            Ok(id.to_string())
        })?
    }

    pub fn package_app_info(&self) -> Result<AppInfo> {
        let app_info = AppInfo::GetFromAppUserModelId(&self.package_app_user_model_id()?.into())?;
        Ok(app_info)
    }

    pub fn program_path(&self) -> Result<PathBuf> {
        let path_string = WindowsApi::exe_path_by_process(self.0)?;
        if path_string.is_empty() {
            return Err("exe path is empty".into());
        }
        Ok(PathBuf::from(path_string))
    }
}
