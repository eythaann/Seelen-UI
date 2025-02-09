use std::path::PathBuf;

use windows::{
    ApplicationModel::AppInfo,
    Win32::{
        Foundation::HANDLE,
        Storage::Packaging::Appx::{
            GetApplicationUserModelId, GetPackageFamilyName, GetPackageFullName,
        },
        System::Threading::{PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION},
    },
};

use crate::error_handler::Result;

use super::{string_utils::WindowsString, types::AppUserModelId, window::Window, WindowsApi};

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
    pub fn from_id(id: u32) -> Self {
        Self(id)
    }

    pub fn from_window(window: &Window) -> Self {
        let (process_id, _) = WindowsApi::window_thread_process_id(window.hwnd());
        Self(process_id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn open_handle(&self) -> Result<HANDLE> {
        WindowsApi::open_process(PROCESS_QUERY_INFORMATION, false, self.0)
    }

    /// will fail if the process is owned by another user
    pub fn open_limited_handle(&self) -> Result<HANDLE> {
        WindowsApi::open_process(PROCESS_QUERY_LIMITED_INFORMATION, false, self.0)
    }

    pub fn is_frozen(&self) -> Result<bool> {
        WindowsApi::is_process_frozen(self.0)
    }

    pub fn package_family_name(&self) -> Result<String> {
        let hprocess = self.open_limited_handle()?;
        let mut len = 1024_u32;
        let mut family_name = WindowsString::new_to_fill(len as usize);
        unsafe { GetPackageFamilyName(hprocess, &mut len, family_name.as_pwstr()).ok()? };
        Ok(family_name.to_string())
    }

    pub fn package_full_name(&self) -> Result<String> {
        let hprocess = self.open_limited_handle()?;
        let mut len = 1024_u32;
        let mut family_name = WindowsString::new_to_fill(len as usize);
        unsafe { GetPackageFullName(hprocess, &mut len, family_name.as_pwstr()).ok()? };
        Ok(family_name.to_string())
    }

    /// package app user model id
    pub fn package_app_user_model_id(&self) -> Result<AppUserModelId> {
        let hprocess = self.open_limited_handle()?;
        let mut len = 1024_u32;
        let mut id = WindowsString::new_to_fill(len as usize);
        unsafe { GetApplicationUserModelId(hprocess, &mut len, id.as_pwstr()).ok()? };
        Ok(AppUserModelId::Appx(id.to_string()))
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

    /// program path filename
    pub fn program_exe_name(&self) -> Result<String> {
        Ok(self
            .program_path()?
            .file_name()
            .ok_or("there is no file name")?
            .to_string_lossy()
            .to_string())
    }

    pub fn program_display_name(&self) -> Result<String> {
        let path = self.program_path()?;
        match WindowsApi::get_executable_display_name(&path) {
            Ok(name) => Ok(name.trim_end_matches(".exe").to_owned()),
            Err(_) => Ok(path
                .file_stem()
                .ok_or("there is no file stem")?
                .to_string_lossy()
                .to_string()),
        }
    }
}
