mod app_bar;
mod com;
mod devices;
pub mod event_window;
pub mod hdc;
mod iterator;
pub mod monitor;
pub mod process;
pub mod string_utils;
pub mod traits;
pub mod types;
pub mod undocumented;
pub mod window;

pub use app_bar::*;
pub use com::*;
pub use devices::*;
pub use iterator::*;

use itertools::Itertools;
use process::ProcessInformationFlag;
use string_utils::WindowsString;
use widestring::U16CStr;
use windows_core::Interface;

use std::{
    ffi::OsString,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

use windows::{
    core::{BSTR, GUID, PCWSTR},
    ApplicationModel::AppInfo,
    Storage::Streams::{
        DataReader, IRandomAccessStreamReference, IRandomAccessStreamWithContentType,
    },
    Wdk::System::{
        SystemServices::PROCESS_EXTENDED_BASIC_INFORMATION,
        Threading::{NtQueryInformationProcess, ProcessBasicInformation},
    },
    Win32::{
        Devices::Display::{
            GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR,
            PHYSICAL_MONITOR,
        },
        Foundation::{
            CloseHandle, HANDLE, HMODULE, HWND, LPARAM, LUID, MAX_PATH, POINT, RECT,
            STATUS_SUCCESS, WPARAM,
        },
        Graphics::{
            Dwm::{
                DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
                DWMWA_VISIBLE_FRAME_BORDER_THICKNESS, DWMWINDOWATTRIBUTE, DWM_CLOAKED_APP,
                DWM_CLOAKED_INHERITED, DWM_CLOAKED_SHELL,
            },
            Gdi::{
                EnumDisplayMonitors, GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow,
                HMONITOR, MONITORENUMPROC, MONITORINFOEXW, MONITOR_DEFAULTTOPRIMARY,
            },
        },
        Security::{
            AdjustTokenPrivileges,
            Authentication::Identity::{GetUserNameExW, EXTENDED_NAME_FORMAT},
            GetTokenInformation, LookupPrivilegeValueW, TokenElevation, TokenLogonSid,
            SE_PRIVILEGE_ENABLED, SE_SHUTDOWN_NAME, TOKEN_ADJUST_PRIVILEGES, TOKEN_ELEVATION,
            TOKEN_GROUPS, TOKEN_PRIVILEGES, TOKEN_QUERY,
        },
        Storage::{
            EnhancedStorage::{
                PKEY_AppUserModel_ID, PKEY_AppUserModel_PreventPinning,
                PKEY_AppUserModel_RelaunchCommand, PKEY_AppUserModel_RelaunchDisplayNameResource,
                PKEY_AppUserModel_RelaunchIconResource, PKEY_AppUserModel_ToastActivatorCLSID,
                PKEY_FileDescription,
            },
            FileSystem::WIN32_FIND_DATAW,
        },
        System::{
            Com::{IPersistFile, STGM_READ},
            Environment::ExpandEnvironmentStringsW,
            LibraryLoader::GetModuleHandleW,
            Power::{GetSystemPowerStatus, SetSuspendState, SYSTEM_POWER_STATUS},
            RemoteDesktop::ProcessIdToSessionId,
            Shutdown::{ExitWindowsEx, LockWorkStation, EXIT_WINDOWS_FLAGS, SHUTDOWN_REASON},
            SystemInformation::{GetComputerNameExW, COMPUTER_NAME_FORMAT},
            Threading::{
                AttachThreadInput, GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId,
                OpenProcess, OpenProcessToken, QueryFullProcessImageNameW, PROCESS_ACCESS_RIGHTS,
                PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::{
            HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
            Shell::{
                BHID_EnumItems, IEnumShellItems, IShellItem2, IShellLinkW, IVirtualDesktopManager,
                PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow, GPS_DEFAULT},
                SHCreateItemFromParsingName, SHGetKnownFolderItem, SHGetKnownFolderPath,
                SHLoadIndirectString, ShellLink, VirtualDesktopManager, KF_FLAG_DEFAULT,
                SIGDN_NORMALDISPLAY,
            },
            WindowsAndMessaging::{
                BringWindowToTop, FindWindowExW, GetClassNameW, GetDesktopWindow,
                GetForegroundWindow, GetParent, GetSystemMetrics, GetWindow, GetWindowLongW,
                GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow,
                IsWindowVisible, IsZoomed, PostMessageW, SetForegroundWindow, SetWindowPos,
                ShowWindow, ShowWindowAsync, SystemParametersInfoW, GWL_EXSTYLE, GWL_STYLE,
                GW_OWNER, SET_WINDOW_POS_FLAGS, SHOW_WINDOW_CMD, SM_CXVIRTUALSCREEN,
                SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SPIF_SENDCHANGE,
                SPIF_UPDATEINIFILE, SPI_GETDESKWALLPAPER, SPI_SETDESKWALLPAPER, SWP_ASYNCWINDOWPOS,
                SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
                WINDOW_EX_STYLE, WINDOW_STYLE, WS_SIZEBOX, WS_THICKFRAME,
            },
        },
    },
    UI::ViewManagement::UISettings,
};

use crate::{
    error::{Result, WindowsResultExt},
    hook::HookManager,
    modules::input::{Keyboard, Mouse},
    windows_api::window::{event::WinEvent, Window},
};

#[macro_export]
macro_rules! pcstr {
    ($s:literal) => {
        windows::core::s!($s)
    };
}

#[macro_export]
macro_rules! pcwstr {
    ($s:literal) => {
        windows::core::w!($s)
    };
}

#[macro_export]
macro_rules! hstring {
    ($s:literal) => {
        windows::core::h!($s)
    };
}

pub struct WindowsApi {}
impl WindowsApi {
    pub fn module_handle_w() -> Result<HMODULE> {
        Ok(unsafe { GetModuleHandleW(None) }?)
    }

    pub fn enum_display_monitors(
        callback: MONITORENUMPROC,
        callback_data_address: isize,
    ) -> Result<()> {
        unsafe {
            EnumDisplayMonitors(None, None, callback, LPARAM(callback_data_address))
                .ok()
                .filter_fake_error()?;
        }
        Ok(())
    }

    pub fn post_message(hwnd: HWND, message: u32, wparam: usize, lparam: isize) -> Result<()> {
        unsafe { PostMessageW(Some(hwnd), message, WPARAM(wparam), LPARAM(lparam))? };
        Ok(())
    }

    pub fn get_monitor_scale_factor(hmonitor: HMONITOR) -> Result<f64> {
        let mut dpi_x: u32 = 0;
        let mut _dpi_y: u32 = 0;
        unsafe { GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut _dpi_y)? };
        // 96 is the default DPI value on Windows
        Ok(dpi_x as f64 / 96_f64)
    }

    pub fn get_text_scale_factor() -> Result<f64> {
        Ok(UISettings::new()?.TextScaleFactor()?)
    }

    /// Behaviour is undefined if an invalid HWND is given
    /// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid
    pub fn window_thread_process_id(hwnd: HWND) -> (u32, u32) {
        let mut process_id: u32 = 0;

        let thread_id = unsafe {
            GetWindowThreadProcessId(hwnd, Option::from(std::ptr::addr_of_mut!(process_id)))
        };

        (process_id, thread_id)
    }

    pub fn find_window(
        parent: Option<HWND>,
        after: Option<HWND>,
        title: Option<String>,
        class: Option<impl Into<String>>,
    ) -> Result<HWND> {
        let title = WindowsString::from(title.unwrap_or_default());
        let class = WindowsString::from(class.map(Into::into).unwrap_or_default());
        let found = unsafe {
            FindWindowExW(
                parent,
                after,
                if class.is_empty() {
                    PCWSTR::null()
                } else {
                    class.as_pcwstr()
                },
                if title.is_empty() {
                    PCWSTR::null()
                } else {
                    title.as_pcwstr()
                },
            )
        }?;
        Ok(found)
    }

    pub fn current_process() -> HANDLE {
        unsafe { GetCurrentProcess() }
    }

    pub fn current_process_id() -> u32 {
        unsafe { GetCurrentProcessId() }
    }

    #[allow(dead_code)]
    pub fn current_thread_id() -> u32 {
        unsafe { GetCurrentThreadId() }
    }

    pub fn current_session_id() -> Result<u32> {
        let process_id = Self::current_process_id();
        let mut session_id = 0;

        unsafe {
            if ProcessIdToSessionId(process_id, &mut session_id).is_ok() {
                Ok(session_id)
            } else {
                Err("could not determine current session id".into())
            }
        }
    }

    /// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow
    pub fn get_foreground_window() -> HWND {
        let mut hwnd = unsafe { GetForegroundWindow() };
        // based on windows doc, get foreground can return null while window is losing activation
        // so we wait until we get a valid window
        while hwnd.is_invalid() {
            hwnd = unsafe { GetForegroundWindow() };
        }
        hwnd
    }

    pub fn is_window(hwnd: HWND) -> bool {
        unsafe { IsWindow(Some(hwnd)) }.into()
    }

    pub fn is_window_visible(hwnd: HWND) -> bool {
        unsafe { IsWindowVisible(hwnd) }.into()
    }

    pub fn is_iconic(hwnd: HWND) -> bool {
        unsafe { IsIconic(hwnd) }.into()
    }

    pub fn is_zoomed(hwnd: HWND) -> bool {
        unsafe { IsZoomed(hwnd) }.into()
    }

    pub fn is_fullscreen(hwnd: HWND) -> Result<bool> {
        let styles = WindowsApi::get_styles(hwnd);
        if styles.contains(WS_THICKFRAME) {
            return Ok(false);
        }

        let rc_monitor = WindowsApi::monitor_rect(WindowsApi::monitor_from_window(hwnd))?;
        let window_rect = WindowsApi::get_inner_window_rect(hwnd)?;
        Ok(window_rect.left <= rc_monitor.left
            && window_rect.top <= rc_monitor.top
            && window_rect.right >= rc_monitor.right
            && window_rect.bottom >= rc_monitor.bottom)
    }

    pub fn is_cloaked(hwnd: HWND) -> Result<bool> {
        let mut cloaked: u32 = 0;
        Self::dwm_get_window_attribute(hwnd, DWMWA_CLOAKED, &mut cloaked)?;
        Ok(matches!(
            cloaked,
            DWM_CLOAKED_APP | DWM_CLOAKED_SHELL | DWM_CLOAKED_INHERITED
        ))
    }

    /// Sets the visibility state of a window created by the calling thread (could cause a deadlock)
    ///
    /// The deadlock occurs if show_window is called for a window created on a different thread but in same process.
    /// Is safe to use for windows created by other processes
    ///
    /// Use this only if you need wait for the window to be visible, otherwise use show_window_async
    ///
    /// https://stackoverflow.com/questions/16881820/win32-api-deadlocks-while-using-different-threads
    /// https://stackoverflow.com/questions/15637124/whats-the-difference-between-showwindow-and-showwindowasync
    pub fn show_window(hwnd: HWND, command: SHOW_WINDOW_CMD) -> Result<()> {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
        unsafe { ShowWindow(hwnd, command) }
            .ok()
            .filter_fake_error()?;
        Ok(())
    }

    pub fn show_window_async(hwnd: HWND, command: SHOW_WINDOW_CMD) -> Result<()> {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindowasync
        unsafe { ShowWindowAsync(hwnd, command) }
            .ok()
            .filter_fake_error()?;
        Ok(())
    }

    pub fn get_styles(hwnd: HWND) -> WINDOW_STYLE {
        WINDOW_STYLE(unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32)
    }

    pub fn get_ex_styles(hwnd: HWND) -> WINDOW_EX_STYLE {
        WINDOW_EX_STYLE(unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32)
    }

    fn _set_position(
        hwnd: HWND,
        order: Option<HWND>,
        rect: RECT,
        flags: SET_WINDOW_POS_FLAGS,
    ) -> Result<()> {
        unsafe {
            SetWindowPos(
                hwnd,
                order,
                rect.left,
                rect.top,
                (rect.right - rect.left).abs(),
                (rect.bottom - rect.top).abs(),
                flags,
            )
            .filter_fake_error()?;
        }
        Ok(())
    }

    /// Similar to ShowWindow could cause a deadlock if the window is created on a different thread.
    ///
    /// Add the flag `SWP_ASYNCWINDOWPOS` to avoid that of if you don't need to wait for the window position to be set
    pub fn set_position(
        hwnd: HWND,
        order: Option<HWND>,
        rect: &RECT,
        flags: SET_WINDOW_POS_FLAGS,
    ) -> Result<()> {
        let flags = match order {
            Some(_) => flags,
            None => SWP_NOZORDER | flags,
        } | SWP_NOACTIVATE;
        Self::_set_position(hwnd, order, *rect, flags)
    }

    pub fn move_window(hwnd: HWND, rect: &RECT) -> Result<()> {
        Self::set_position(hwnd, None, rect, SWP_NOSIZE | SWP_ASYNCWINDOWPOS)
    }

    #[allow(dead_code)]
    pub fn bring_to_top(hwnd: HWND) -> Result<()> {
        unsafe { BringWindowToTop(hwnd)? };
        Ok(())
    }

    #[allow(dead_code)]
    pub fn attach_thread_input(thread_id: u32, attach_to: u32, attach: bool) -> Result<()> {
        unsafe { AttachThreadInput(thread_id, attach_to, attach).ok()? };
        Ok(())
    }

    pub fn set_foreground(hwnd: HWND) -> Result<()> {
        let window = Window::from(hwnd);

        if !unsafe { SetForegroundWindow(hwnd).as_bool() } {
            // https://stackoverflow.com/questions/10740346/setforegroundwindow-only-working-while-visual-studio-is-open
            let keyboard = Keyboard::new();
            keyboard.send_keys("{alt}")?;
            // this can fail but still be successful.
            let _ = unsafe { SetForegroundWindow(hwnd) };
        }

        // extra validation
        if Window::get_foregrounded() != window {
            return Err("Failed to set foreground window".into());
        }

        // event sometimes is not emitted, so we manually emit it, this will cause 2 foreground events
        // if original was recieved, btw having it twice is better than nothing
        HookManager::event_tx().send((WinEvent::SystemForeground, window))?;
        Ok(())
    }

    fn open_process(
        access_rights: PROCESS_ACCESS_RIGHTS,
        inherit_handle: bool,
        process_id: u32,
    ) -> Result<HANDLE> {
        unsafe { Ok(OpenProcess(access_rights, inherit_handle, process_id)?) }
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

    #[allow(dead_code)]
    pub fn get_current_process_info() -> Result<()> {
        let token_handle = Self::open_current_process_token()?;
        let mut returnlength = 0;
        unsafe {
            let data = TOKEN_GROUPS::default();

            GetTokenInformation(
                token_handle,
                TokenLogonSid,
                Some(&data as *const _ as *mut _),
                std::mem::size_of::<TOKEN_GROUPS>() as u32,
                &mut returnlength,
            )?;
        }
        Ok(())
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

        unsafe { AdjustTokenPrivileges(token_handle, false, Some(&tkp), 0, None, None)? };
        Ok(())
    }

    pub fn get_parent(hwnd: HWND) -> Result<HWND> {
        Ok(unsafe { GetParent(hwnd)? })
    }

    pub fn get_owner(hwnd: HWND) -> Result<HWND> {
        Ok(unsafe { GetWindow(hwnd, GW_OWNER)? })
    }

    pub fn get_desktop_window() -> HWND {
        unsafe { GetDesktopWindow() }
    }

    pub fn is_process_frozen(process_id: u32) -> Result<bool> {
        let handle = Self::open_process(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)?;
        let is_frozen = unsafe {
            let mut buffer: [PROCESS_EXTENDED_BASIC_INFORMATION; 1] = std::mem::zeroed();
            let status = NtQueryInformationProcess(
                handle,
                ProcessBasicInformation,
                buffer.as_mut_ptr() as _,
                std::mem::size_of::<PROCESS_EXTENDED_BASIC_INFORMATION>() as _,
                0u32 as _,
            );

            if status != STATUS_SUCCESS {
                return Err(format!(
                    "NtQueryInformationProcess failed with status: {:x}",
                    status.0
                )
                .into());
            }

            let data = buffer[0];
            data.Anonymous.Flags & ProcessInformationFlag::IsFrozen as u32 != 0
        };
        Ok(is_frozen)
    }

    pub fn exe_path_by_process(process_id: u32) -> Result<OsString> {
        let mut path = WindowsString::new_to_fill(1024);
        let handle = Self::open_process(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)?;
        unsafe {
            QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, path.as_pwstr(), &mut 1024)?;
        }
        Ok(path.to_os_string())
    }

    pub fn get_class(hwnd: HWND) -> Result<String> {
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetClassNameW(hwnd, &mut text) };
        let length = usize::try_from(len).unwrap_or(0);
        Ok(String::from_utf16(&text[..length])?)
    }

    pub fn get_shell_item(path: &Path) -> Result<IShellItem2> {
        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let item = unsafe { SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None)? };
        Ok(item)
    }

    pub fn get_property_store_for_window(hwnd: HWND) -> Result<IPropertyStore> {
        Ok(unsafe { SHGetPropertyStoreForWindow(hwnd)? })
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-id
    pub fn get_window_app_user_model_id(hwnd: HWND) -> Result<String> {
        let store = Self::get_property_store_for_window(hwnd)?;
        let value = unsafe { store.GetValue(&PKEY_AppUserModel_ID)? };
        if value.is_empty() {
            return Err("No AppUserModel_ID".into());
        }
        Ok(BSTR::try_from(&value)?.to_string())
    }

    pub fn get_window_prevent_pinning(hwnd: HWND) -> Result<bool> {
        let store = Self::get_property_store_for_window(hwnd)?;
        let value = unsafe { store.GetValue(&PKEY_AppUserModel_PreventPinning)? };
        if value.is_empty() {
            return Err("No AppUserModel_PreventPinning".into());
        }
        Ok(bool::try_from(&value)?)
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchcommand
    pub fn get_window_relaunch_command(hwnd: HWND) -> Result<String> {
        let store = Self::get_property_store_for_window(hwnd)?;
        let value = unsafe { store.GetValue(&PKEY_AppUserModel_RelaunchCommand)? };
        if value.is_empty() {
            return Err("No AppUserModel_RelaunchCommand".into());
        }
        Ok(BSTR::try_from(&value)?.to_string())
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchdisplaynameresource
    pub fn get_window_relaunch_display_name(hwnd: HWND) -> Result<String> {
        let store = Self::get_property_store_for_window(hwnd)?;
        let value = unsafe { store.GetValue(&PKEY_AppUserModel_RelaunchDisplayNameResource)? };
        if value.is_empty() {
            return Err("No AppUserModel_RelaunchDisplayName".into());
        }
        Ok(BSTR::try_from(&value)?.to_string())
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchiconresource
    pub fn get_window_relaunch_icon_resource(hwnd: HWND) -> Result<String> {
        let store = Self::get_property_store_for_window(hwnd)?;
        let value = unsafe { store.GetValue(&PKEY_AppUserModel_RelaunchIconResource)? };
        if value.is_empty() {
            return Err("No AppUserModel_RelaunchIconResource".into());
        }
        Ok(BSTR::try_from(&value)?.to_string())
    }

    pub fn is_uwp_package_id(package_id: &str) -> bool {
        Self::get_uwp_app_info(package_id).is_ok()
    }

    pub fn get_uwp_app_info(umid: &str) -> Result<AppInfo> {
        let app_info = AppInfo::GetFromAppUserModelId(&umid.into())?;
        Ok(app_info)
    }

    pub fn create_temp_shortcut(
        program: &Path,
        args: &str,
        working_dir: Option<&Path>,
    ) -> Result<PathBuf> {
        let working_dir = working_dir.or_else(|| program.parent());

        Com::run_with_context(|| unsafe {
            let shell_link: IShellLinkW = Com::create_instance(&ShellLink)?;

            let program = WindowsString::from_os_string(program.as_os_str());
            shell_link.SetPath(program.as_pcwstr())?;

            let arguments = WindowsString::from_str(args);
            shell_link.SetArguments(arguments.as_pcwstr())?;

            if let Some(working_dir) = working_dir {
                let working_dir = WindowsString::from_os_string(working_dir.as_os_str());
                shell_link.SetWorkingDirectory(working_dir.as_pcwstr())?;
            }

            let temp_dir = std::env::temp_dir();
            let lnk_path = temp_dir.join(format!("{}.lnk", uuid::Uuid::new_v4()));
            let lnk_path_wide = WindowsString::from_os_string(lnk_path.as_os_str());

            let persist_file: IPersistFile = shell_link.cast()?;
            persist_file.Save(lnk_path_wide.as_pcwstr(), true)?;
            Ok(lnk_path)
        })
    }

    /// return the program and arguments
    pub fn resolve_lnk_target(lnk_path: &Path) -> Result<(PathBuf, OsString)> {
        Com::run_with_context(|| {
            let shell_link: IShellLinkW = Com::create_instance(&ShellLink)?;
            let lnk_wide = lnk_path
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect_vec();

            let persist_file: IPersistFile = shell_link.cast()?;
            unsafe { persist_file.Load(PCWSTR(lnk_wide.as_ptr()), STGM_READ)? };

            let mut target_path = WindowsString::new_to_fill(1024);
            let mut idk = WIN32_FIND_DATAW::default();
            unsafe { shell_link.GetPath(target_path.as_mut_slice(), &mut idk, 0)? };
            target_path = Self::resolve_environment_variables(&target_path)?;

            let mut arguments = WindowsString::new_to_fill(1024);
            unsafe { shell_link.GetArguments(arguments.as_mut_slice())? };

            Ok((target_path.to_os_string().into(), arguments.to_os_string()))
        })
    }

    pub fn resolve_lnk_custom_icon_path(lnk_path: &Path) -> Result<PathBuf> {
        Com::run_with_context(|| {
            let shell_link: IShellLinkW = Com::create_instance(&ShellLink)?;
            let lnk_wide = lnk_path
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect_vec();

            let persist_file: IPersistFile = shell_link.cast()?;
            unsafe { persist_file.Load(PCWSTR(lnk_wide.as_ptr()), STGM_READ)? };

            let mut icon_path = WindowsString::new_to_fill(1024);
            let mut icon_idx = 0;
            unsafe { shell_link.GetIconLocation(icon_path.as_mut_slice(), &mut icon_idx)? };

            if icon_path.is_empty() {
                return Err("There is no custom icon for this link file".into());
            }

            icon_path = Self::resolve_environment_variables(&icon_path)?;
            Ok(PathBuf::from(icon_path.to_os_string()))
        })
    }

    /// https://learn.microsoft.com/en-us/windows/win32/api/shlwapi/nf-shlwapi-shloadindirectstring
    /// Extracts a specified text resource when given that resource in the form of an indirect string
    /// (a string that begins with the '@' symbol).
    pub fn resolve_indirect_string(text: &str) -> Result<String> {
        let source = WindowsString::from_str(text);
        let mut out = WindowsString::new_to_fill(1024);
        unsafe { SHLoadIndirectString(source.as_pcwstr(), out.as_mut_slice(), None)? };
        Ok(out.to_string())
    }

    /// https://learn.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-expandenvironmentstringsw
    /// Expands all environment variables in a string (for example, %PATH%).
    pub fn resolve_environment_variables(source: &WindowsString) -> Result<WindowsString> {
        let len = unsafe { ExpandEnvironmentStringsW(source.as_pcwstr(), None) };
        let mut out = WindowsString::new_to_fill(len as usize);
        unsafe { ExpandEnvironmentStringsW(source.as_pcwstr(), Some(out.as_mut_slice())) };
        Ok(out)
    }

    pub fn get_executable_display_name(path: &Path) -> Result<String> {
        Com::run_with_context(|| unsafe {
            let shell_item = Self::get_shell_item(path)?;
            let text = shell_item
                .GetString(&PKEY_FileDescription)
                .or_else(|_| shell_item.GetDisplayName(SIGDN_NORMALDISPLAY))?;
            Ok(text.to_string()?)
        })
    }

    pub fn get_file_umid(path: &Path) -> Result<String> {
        Com::run_with_context(|| unsafe {
            let shell_item = Self::get_shell_item(path)?;
            let store: IPropertyStore = shell_item.GetPropertyStore(GPS_DEFAULT)?;
            let value = store.GetValue(&PKEY_AppUserModel_ID)?;
            if value.is_empty() {
                return Err("No AppUserModel_ID".into());
            }
            Ok(value.to_string())
        })
    }

    pub fn get_file_toast_activator(path: &Path) -> Result<String> {
        Com::run_with_context(|| unsafe {
            let shell_item = Self::get_shell_item(path)?;
            let store: IPropertyStore = shell_item.GetPropertyStore(GPS_DEFAULT)?;
            let value = store.GetValue(&PKEY_AppUserModel_ToastActivatorCLSID)?;
            if value.is_empty() {
                return Err("No AppUserModel ToastActivator CLSID".into());
            }
            Ok(value
                .to_string()
                .trim_start_matches("{")
                .trim_end_matches("}")
                .to_owned())
        })
    }

    pub fn get_window_text(hwnd: HWND) -> String {
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetWindowTextW(hwnd, &mut text) };
        let length = usize::try_from(len).unwrap_or(0);
        String::from_utf16(&text[..length]).unwrap_or("".to_owned())
    }

    pub fn dwm_get_window_attribute<T>(
        hwnd: HWND,
        attribute: DWMWINDOWATTRIBUTE,
        value: &mut T,
    ) -> Result<()> {
        unsafe {
            DwmGetWindowAttribute(
                hwnd,
                attribute,
                (value as *mut T).cast(),
                u32::try_from(std::mem::size_of::<T>())?,
            )?;
        }
        Ok(())
    }

    /// Get the window rect including drop shadow
    pub fn get_outer_window_rect(hwnd: HWND) -> Result<RECT> {
        let mut rect = RECT::default();
        unsafe { GetWindowRect(hwnd, &mut rect)? };
        Ok(rect)
    }

    fn get_window_thickness(hwnd: HWND) -> u32 {
        let mut thickness = 0u32;
        let _ = Self::dwm_get_window_attribute(
            hwnd,
            DWMWA_VISIBLE_FRAME_BORDER_THICKNESS,
            &mut thickness,
        );
        thickness
    }

    /// return the window rect excluding drop shadow & thick border
    /// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowrect#remarks
    pub fn get_inner_window_rect(hwnd: HWND) -> Result<RECT> {
        let mut rect = RECT::default();
        if Self::dwm_get_window_attribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &mut rect).is_err() {
            rect = Self::get_outer_window_rect(hwnd)?;
        }

        let styles = Self::get_styles(hwnd);
        if styles.contains(WS_THICKFRAME) || styles.contains(WS_SIZEBOX) {
            let thickness = Self::get_window_thickness(hwnd) as i32;
            rect.left += thickness;
            rect.top += thickness;
            rect.right -= thickness;
            rect.bottom -= thickness;
        }

        Ok(rect)
    }

    pub fn monitor_from_window(hwnd: HWND) -> HMONITOR {
        unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) }
    }

    pub fn monitor_from_cursor_point() -> HMONITOR {
        if let Ok(point) = Mouse::get_cursor_pos() {
            return unsafe {
                MonitorFromPoint(
                    POINT {
                        x: point.x,
                        y: point.y,
                    },
                    MONITOR_DEFAULTTOPRIMARY,
                )
            };
        }
        Self::primary_monitor()
    }

    pub fn monitor_from_point(point: &seelen_core::Point) -> HMONITOR {
        unsafe {
            MonitorFromPoint(
                POINT {
                    x: point.x,
                    y: point.y,
                },
                MONITOR_DEFAULTTOPRIMARY,
            )
        }
    }

    pub fn primary_monitor() -> HMONITOR {
        unsafe { MonitorFromWindow(GetDesktopWindow(), MONITOR_DEFAULTTOPRIMARY) }
    }

    pub fn get_physical_monitors(monitor: HMONITOR) -> Result<Vec<PHYSICAL_MONITOR>> {
        let mut c_physical_monitors: u32 = 0;
        let mut p_physical_monitors: Vec<PHYSICAL_MONITOR> = Vec::new();
        unsafe {
            GetNumberOfPhysicalMonitorsFromHMONITOR(monitor, &mut c_physical_monitors)?;
            p_physical_monitors.resize(c_physical_monitors as usize, std::mem::zeroed());
            GetPhysicalMonitorsFromHMONITOR(monitor, p_physical_monitors.as_mut())?;
        };
        Ok(p_physical_monitors)
    }

    /// https://learn.microsoft.com/en-us/windows/win32/gdi/the-virtual-screen
    pub fn virtual_screen_rect() -> Result<RECT> {
        let mut rect = RECT::default();
        unsafe {
            rect.left = GetSystemMetrics(SM_XVIRTUALSCREEN);
            rect.top = GetSystemMetrics(SM_YVIRTUALSCREEN);
            rect.right = rect.left + GetSystemMetrics(SM_CXVIRTUALSCREEN);
            rect.bottom = rect.top + GetSystemMetrics(SM_CYVIRTUALSCREEN);
        }
        Ok(rect)
    }

    pub fn monitor_info(hmonitor: HMONITOR) -> Result<MONITORINFOEXW> {
        let mut ex_info = MONITORINFOEXW::default();
        ex_info.monitorInfo.cbSize = u32::try_from(std::mem::size_of::<MONITORINFOEXW>())?;
        unsafe { GetMonitorInfoW(hmonitor, &mut ex_info.monitorInfo).ok() }?;
        Ok(ex_info)
    }

    pub fn monitor_rect(hmonitor: HMONITOR) -> Result<RECT> {
        Ok(Self::monitor_info(hmonitor)?.monitorInfo.rcMonitor)
    }

    pub fn shadow_rect(hwnd: HWND) -> Result<RECT> {
        let outer_rect = Self::get_outer_window_rect(hwnd)?;
        let inner_rect = Self::get_inner_window_rect(hwnd)?;
        Ok(RECT {
            left: outer_rect.left - inner_rect.left,
            top: outer_rect.top - inner_rect.top,
            right: outer_rect.right - inner_rect.right,
            bottom: outer_rect.bottom - inner_rect.bottom,
        })
    }

    pub fn _get_virtual_desktop_manager() -> Result<IVirtualDesktopManager> {
        Com::create_instance(&VirtualDesktopManager)
    }

    pub fn _get_virtual_desktop_id(hwnd: HWND) -> Result<GUID> {
        let manager = Self::_get_virtual_desktop_manager()?;
        let mut desktop_id = GUID::zeroed();
        let mut attempt = 0;
        while desktop_id.to_u128() == 0 && attempt < 10 {
            attempt += 1;
            sleep(Duration::from_millis(30));
            if let Ok(desktop) = unsafe { manager.GetWindowDesktopId(hwnd) } {
                desktop_id = desktop
            }
        }
        if desktop_id.to_u128() == 0 {
            return Err(format!("Failed to get desktop id for: {hwnd:?}").into());
        }
        Ok(desktop_id)
    }

    pub fn get_wallpaper() -> Result<PathBuf> {
        let mut path = [0_u16; MAX_PATH as usize];
        unsafe {
            SystemParametersInfoW(
                SPI_GETDESKWALLPAPER,
                MAX_PATH,
                Some(path.as_mut_ptr() as _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )?;
        }
        Ok(PathBuf::from(
            U16CStr::from_slice_truncate(&path)?
                .to_ustring()
                .to_string_lossy(),
        ))
    }

    pub fn refresh_desktop() -> Result<()> {
        unsafe {
            SystemParametersInfoW(SPI_SETDESKWALLPAPER, 0, None, SPIF_UPDATEINIFILE)?;
        }
        Ok(())
    }

    pub fn set_wallpaper(path: String) -> Result<()> {
        if !PathBuf::from(&path).exists() {
            return Err("File not found".into());
        }

        let mut path = path.encode_utf16().chain(Some(0)).collect_vec();
        unsafe {
            SystemParametersInfoW(
                SPI_SETDESKWALLPAPER,
                MAX_PATH,
                Some(path.as_mut_ptr() as _),
                SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
            )?;
        }
        Ok(())
    }

    /// https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-exitwindowsex
    pub fn exit_windows(flags: EXIT_WINDOWS_FLAGS, reason: SHUTDOWN_REASON) -> Result<()> {
        WindowsApi::enable_privilege(SE_SHUTDOWN_NAME)?;
        unsafe { ExitWindowsEx(flags, reason) }?;
        Ok(())
    }

    /// https://learn.microsoft.com/en-us/windows/win32/api/powrprof/nf-powrprof-setsuspendstate
    pub fn set_suspend_state(hibernate: bool) -> Result<()> {
        WindowsApi::enable_privilege(SE_SHUTDOWN_NAME)?;
        let success = unsafe { SetSuspendState(hibernate, false, false) };
        if !success {
            return Err("Failed to set suspend state".into());
        }
        Ok(())
    }

    pub fn is_elevated() -> Result<bool> {
        unsafe {
            let mut elevation = TOKEN_ELEVATION::default();
            let mut ret_len = 0;

            let token_handle = Self::open_current_process_token()?;

            GetTokenInformation(
                token_handle,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut ret_len,
            )?;

            CloseHandle(token_handle)?;

            Ok(elevation.TokenIsElevated != 0)
        }
    }

    pub fn get_system_power_status() -> Result<SYSTEM_POWER_STATUS> {
        let mut power_status = SYSTEM_POWER_STATUS::default();
        unsafe {
            GetSystemPowerStatus(&mut power_status as _)?;
        }
        Ok(power_status)
    }

    pub fn stream_to_dynamic_image(
        stream: IRandomAccessStreamWithContentType,
    ) -> Result<image::DynamicImage> {
        let size = stream.Size()?;
        let mut buffer = vec![0u8; size as usize];

        let input_stream = stream.GetInputStreamAt(0)?;
        let data_reader = DataReader::CreateDataReader(&input_stream)?;

        data_reader.LoadAsync(size as u32)?.get()?;
        data_reader.ReadBytes(&mut buffer)?;

        let image = image::load_from_memory_with_format(&buffer, image::ImageFormat::Png)?;
        Ok(image)
    }

    pub fn extract_thumbnail_from_stream(
        stream: IRandomAccessStreamWithContentType,
    ) -> Result<PathBuf> {
        let image = Self::stream_to_dynamic_image(stream)?;
        let image_path = std::env::temp_dir().join(format!("{}.png", uuid::Uuid::new_v4()));
        image.save(&image_path)?;
        Ok(image_path)
    }

    pub fn extract_thumbnail_from_ref(stream: IRandomAccessStreamReference) -> Result<PathBuf> {
        Self::extract_thumbnail_from_stream(stream.OpenReadAsync()?.get()?)
    }

    pub fn lock_machine() -> Result<()> {
        unsafe { Ok(LockWorkStation()?) }
    }

    // get current thread owner username
    pub fn get_username(format: EXTENDED_NAME_FORMAT) -> Result<String> {
        let mut size = 0;
        unsafe { GetUserNameExW(format, None, &mut size) };
        let mut name = WindowsString::new_to_fill(size as usize);
        let sucess = unsafe { GetUserNameExW(format, Some(name.as_pwstr()), &mut size) };
        if !sucess {
            return Err("Failed to get username".into());
        }
        Ok(name.to_string())
    }

    pub fn get_computer_name(format: COMPUTER_NAME_FORMAT) -> Result<String> {
        let mut name = WindowsString::new_to_fill(1024);
        unsafe { GetComputerNameExW(format, Some(name.as_pwstr()), &mut 1024)? };
        Ok(name.to_string())
    }

    /// https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid
    pub fn known_folder(folder_id: windows::core::GUID) -> Result<PathBuf> {
        let path = unsafe { SHGetKnownFolderPath(&folder_id, KF_FLAG_DEFAULT, None)? };
        Ok(PathBuf::from(OsString::from_wide(unsafe {
            path.as_wide()
        })))
    }

    #[allow(dead_code)]
    pub fn known_folder_item(folder_id: windows::core::GUID) -> Result<()> {
        let item: IShellItem2 = unsafe { SHGetKnownFolderItem(&folder_id, KF_FLAG_DEFAULT, None)? };
        let enumerator: IEnumShellItems = unsafe { item.BindToHandler(None, &BHID_EnumItems)? };

        loop {
            let mut items = [None; 1];
            let mut fetched = 0u32;

            unsafe { enumerator.Next(&mut items, Some(&mut fetched))? };

            if fetched == 0 {
                break;
            }

            if let Some(item) = &items[0] {
                let item: IShellItem2 = item.cast()?;
                // Obtener el nombre para mostrar
                let display_name = unsafe {
                    item.GetDisplayName(SIGDN_NORMALDISPLAY)?
                        .to_hstring()
                        .to_os_string()
                };
                let umid = unsafe {
                    item.GetString(&PKEY_AppUserModel_ID)
                        .ok()
                        .map(|s| s.to_hstring().to_os_string())
                };
                println!("- {} // {:?}", display_name.display(), umid);
            }
        }

        Ok(())
    }
}
