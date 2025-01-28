pub mod com;

use std::path::PathBuf;

use com::Com;
use windows::Win32::{
    Foundation::{FALSE, HANDLE, HWND, LUID},
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    },
    System::{
        Com::IPersistFile,
        Threading::{GetCurrentProcess, OpenProcessToken},
    },
    UI::{
        HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        Shell::{IShellLinkW, ShellLink},
        WindowsAndMessaging::{
            SetWindowPos, ShowWindow, ShowWindowAsync, SET_WINDOW_POS_FLAGS, SHOW_WINDOW_CMD,
            SWP_NOZORDER,
        },
    },
};
use windows_core::{Interface, PCWSTR};

use crate::{
    error::{Result, WindowsResultExt},
    string_utils::WindowsString,
};

pub struct WindowsApi;

impl WindowsApi {
    pub fn show_window(hwnd: isize, command: i32) -> Result<()> {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
        unsafe { ShowWindow(HWND(hwnd as _), SHOW_WINDOW_CMD(command)) }
            .ok()
            .filter_fake_error()?;
        Ok(())
    }

    pub fn show_window_async(hwnd: isize, command: i32) -> Result<()> {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindowasync
        unsafe { ShowWindowAsync(HWND(hwnd as _), SHOW_WINDOW_CMD(command)) }
            .ok()
            .filter_fake_error()?;
        Ok(())
    }

    pub fn set_position(
        hwnd: isize,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    ) -> Result<()> {
        unsafe {
            SetWindowPos(
                HWND(hwnd as _),
                HWND::default(),
                x,
                y,
                width,
                height,
                SET_WINDOW_POS_FLAGS(flags) | SWP_NOZORDER,
            )
            .filter_fake_error()?;
        }
        Ok(())
    }

    pub fn set_process_dpi_aware() -> Result<()> {
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)? };
        Ok(())
    }

    pub fn current_process() -> HANDLE {
        unsafe { GetCurrentProcess() }
    }

    pub fn open_current_process_token() -> Result<HANDLE> {
        let mut token_handle = HANDLE::default();
        unsafe {
            OpenProcessToken(
                Self::current_process(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token_handle,
            )?;
        }
        if token_handle.is_invalid() {
            return Err("OpenProcessToken failed".into());
        }
        Ok(token_handle)
    }

    pub fn get_luid(system: PCWSTR, name: PCWSTR) -> Result<LUID> {
        let mut luid = LUID::default();
        unsafe { LookupPrivilegeValueW(system, name, &mut luid)? };
        Ok(luid)
    }

    pub fn enable_privilege(name: PCWSTR) -> Result<()> {
        let token_handle = Self::open_current_process_token()?;
        let mut tkp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            ..Default::default()
        };

        tkp.Privileges[0].Luid = Self::get_luid(PCWSTR::null(), name)?;
        tkp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        unsafe { AdjustTokenPrivileges(token_handle, FALSE, Some(&tkp), 0, None, None)? };
        Ok(())
    }

    pub fn create_temp_shortcut(program: &str, args: &str) -> Result<PathBuf> {
        Com::run_with_context(|| unsafe {
            let shell_link: IShellLinkW = Com::create_instance(&ShellLink)?;

            let program = WindowsString::from_str(program);
            shell_link.SetPath(program.as_pcwstr())?;

            let arguments = WindowsString::from_str(args);
            shell_link.SetArguments(arguments.as_pcwstr())?;

            let temp_dir = std::env::temp_dir();
            let lnk_path = temp_dir.join(format!("{}.lnk", uuid::Uuid::new_v4()));
            let lnk_path_wide = WindowsString::from_os_string(lnk_path.as_os_str());

            let persist_file: IPersistFile = shell_link.cast()?;
            persist_file.Save(lnk_path_wide.as_pcwstr(), true)?;
            Ok(lnk_path)
        })
    }
}
