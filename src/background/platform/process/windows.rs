use std::path::PathBuf;

use windows::{
    ApplicationModel::AppInfo,
    Win32::{
        Foundation::HANDLE,
        Storage::Packaging::Appx::GetApplicationUserModelId,
        System::Threading::{PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION},
    },
};
use windows_core::Owned;

use crate::{
    error::Result,
    windows_api::{WindowsApi, string_utils::WindowsString, types::AppUserModelId},
};

use super::Process;

pub struct NativeProcess(u32);

impl NativeProcess {
    pub fn from_id(id: u32) -> Self {
        Self(id)
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn exe_path(&self) -> Result<PathBuf> {
        let path_string = WindowsApi::exe_path_by_process(self.0)?;
        if path_string.is_empty() {
            return Err("exe path is empty".into());
        }
        Ok(PathBuf::from(path_string))
    }

    pub fn display_name(&self) -> Result<String> {
        let path = self.exe_path()?;
        match WindowsApi::get_executable_display_name(&path) {
            Ok(name) => Ok(name.trim_end_matches(".exe").to_owned()),
            Err(_) => Ok(path
                .file_stem()
                .ok_or("there is no file stem")?
                .to_string_lossy()
                .to_string()),
        }
    }

    pub fn is_frozen(&self) -> Result<bool> {
        WindowsApi::is_process_frozen(self.0)
    }

    pub fn kill(&self) -> Result<()> {
        use std::os::windows::process::CommandExt;
        use windows::Win32::System::Threading::CREATE_NO_WINDOW;

        std::process::Command::new("taskkill.exe")
            .args(["/F", "/PID", &self.0.to_string()])
            .creation_flags(CREATE_NO_WINDOW.0)
            .spawn()?;
        Ok(())
    }
}

/// Windows only concepts of a process, use it only from Windows specific code.
pub trait WindowsProcessExt {
    fn open_handle(&self) -> Result<Owned<HANDLE>>;
    /// will fail if the process is owned by another user
    fn open_limited_handle(&self) -> Result<Owned<HANDLE>>;
    /// package app user model id, (appx, eg: "Microsoft.WindowsTerminal_8wekyb3d8bbwe!TerminalApp")
    fn package_app_user_model_id(&self) -> Result<AppUserModelId>;
    #[allow(dead_code)]
    fn package_app_info(&self) -> Result<AppInfo>;
}

impl WindowsProcessExt for Process {
    fn open_handle(&self) -> Result<Owned<HANDLE>> {
        WindowsApi::open_process(PROCESS_QUERY_INFORMATION, false, self.id())
    }

    fn open_limited_handle(&self) -> Result<Owned<HANDLE>> {
        WindowsApi::open_process(PROCESS_QUERY_LIMITED_INFORMATION, false, self.id())
    }

    fn package_app_user_model_id(&self) -> Result<AppUserModelId> {
        let hprocess = self.open_limited_handle()?;
        let mut len = 1024_u32;
        let mut id = WindowsString::new_to_fill(len as usize);
        unsafe { GetApplicationUserModelId(*hprocess, &mut len, Some(id.as_pwstr())).ok()? };
        Ok(AppUserModelId::Appx(id.to_string()))
    }

    fn package_app_info(&self) -> Result<AppInfo> {
        let app_info = AppInfo::GetFromAppUserModelId(&self.package_app_user_model_id()?.into())?;
        Ok(app_info)
    }
}
