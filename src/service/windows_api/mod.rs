pub mod app_bar;
pub mod com;
pub mod iterator;

use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf};

use windows::Win32::{
    Foundation::{HANDLE, HWND, LUID},
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    },
    System::{
        Console::GetConsoleWindow,
        RemoteDesktop::ProcessIdToSessionId,
        Threading::{GetCurrentProcess, GetCurrentProcessId, OpenProcessToken},
    },
    UI::{
        HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        Input::KeyboardAndMouse::{keybd_event, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_MENU},
        Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT},
        WindowsAndMessaging::{
            FindWindowW, GetClassNameW, GetForegroundWindow, GetWindowTextW, SetForegroundWindow,
            SetWindowPos, ShowWindow, ShowWindowAsync, SET_WINDOW_POS_FLAGS, SHOW_WINDOW_CMD,
            SWP_NOACTIVATE, SWP_NOZORDER,
        },
    },
};
use windows_core::{Owned, PCWSTR};

use crate::{
    error::{Result, WindowsResultExt},
    string_utils::WindowsString,
};

pub struct WindowsApi;

impl WindowsApi {
    pub fn show_window(addr: isize, command: i32) -> Result<()> {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
        unsafe { ShowWindow(HWND(addr as _), SHOW_WINDOW_CMD(command)) }
            .ok()
            .filter_fake_error()?;
        Ok(())
    }

    pub fn show_window_async(addr: isize, command: i32) -> Result<()> {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindowasync
        unsafe { ShowWindowAsync(HWND(addr as _), SHOW_WINDOW_CMD(command)) }
            .ok()
            .filter_fake_error()?;
        Ok(())
    }

    pub fn get_foreground_window() -> HWND {
        unsafe { GetForegroundWindow() }
    }

    pub fn set_foreground(addr: isize) -> Result<()> {
        let target_hwnd = HWND(addr as _);
        if !unsafe { SetForegroundWindow(target_hwnd).as_bool() } {
            // https://stackoverflow.com/questions/10740346/setforegroundwindow-only-working-while-visual-studio-is-open
            unsafe {
                keybd_event(VK_MENU.0 as u8, 0x45, KEYEVENTF_EXTENDEDKEY, 0);
                keybd_event(
                    VK_MENU.0 as u8,
                    0x45,
                    KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP,
                    0,
                );
            }
            // this can fail but still be successful.
            let _ = unsafe { SetForegroundWindow(target_hwnd) };
        }

        // based on windows doc, get foreground can return null while window is losing activation
        // so we wait until we get a valid window.
        let mut focus_hwnd = Self::get_foreground_window();
        let mut retries = 0;
        while focus_hwnd != target_hwnd && retries < 10 {
            std::thread::sleep(std::time::Duration::from_millis(1));
            focus_hwnd = Self::get_foreground_window();
            retries += 1;
        }

        if focus_hwnd != target_hwnd {
            return Err("Failed to set foreground window".into());
        }
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
                None,
                x,
                y,
                width,
                height,
                SET_WINDOW_POS_FLAGS(flags) | SWP_NOACTIVATE | SWP_NOZORDER,
            )
            .filter_fake_error()?;
        }
        Ok(())
    }

    pub fn set_process_dpi_aware() -> Result<()> {
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)? };
        Ok(())
    }

    pub fn get_console_window() -> HWND {
        unsafe { GetConsoleWindow() }
    }

    pub fn current_process() -> HANDLE {
        unsafe { GetCurrentProcess() }
    }

    pub fn current_process_id() -> u32 {
        unsafe { GetCurrentProcessId() }
    }

    pub fn current_session_id() -> u32 {
        let process_id = Self::current_process_id();
        let mut session_id = 0;
        // this should never fail for own process
        unsafe { ProcessIdToSessionId(process_id, &mut session_id).expect("Can't get session id") };
        session_id
    }

    pub fn open_current_process_token() -> Result<Owned<HANDLE>> {
        let mut token_handle = HANDLE::default();
        unsafe {
            OpenProcessToken(
                Self::current_process(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token_handle,
            )?;

            if token_handle.is_invalid() {
                return Err("OpenProcessToken failed".into());
            }
            Ok(Owned::new(token_handle))
        }
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

        unsafe { AdjustTokenPrivileges(*token_handle, false, Some(&tkp), 0, None, None)? };
        Ok(())
    }

    // change to some crate like dirs to allow multiple platforms
    pub fn known_folder(folder_id: windows::core::GUID) -> Result<PathBuf> {
        let path = unsafe { SHGetKnownFolderPath(&folder_id, KF_FLAG_DEFAULT, None)? };
        Ok(PathBuf::from(OsString::from_wide(unsafe {
            path.as_wide()
        })))
    }

    pub fn get_class(hwnd: HWND) -> String {
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetClassNameW(hwnd, &mut text) };
        let length = usize::try_from(len).unwrap_or(0);
        String::from_utf16_lossy(&text[..length])
    }

    pub fn get_title(hwnd: HWND) -> String {
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetWindowTextW(hwnd, &mut text) };
        let length = usize::try_from(len).unwrap_or(0);
        String::from_utf16_lossy(&text[..length])
    }

    pub fn wait_for_native_shell() {
        log::info!("Waiting for native shell...");
        let mut attempt = 0;
        let class = WindowsString::from_str("Shell_TrayWnd");
        unsafe {
            // wait for native shell until 50 attempts or 5 seconds
            while FindWindowW(class.as_pcwstr(), None).is_err() && attempt < 50 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                attempt += 1;
            }
        }
        if attempt >= 50 {
            log::warn!("Native shell not found after 5 seconds, continuing anyway...");
        }
        log::info!("Native shell found, continueing setup...");
    }
}
