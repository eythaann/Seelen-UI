use std::{ffi::c_void, thread::sleep, time::Duration};

use color_eyre::eyre::eyre;
use windows::{
    core::{GUID, PCWSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HANDLE, HWND, RECT},
        Graphics::{
            Dwm::{
                DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
                DWMWINDOWATTRIBUTE, DWM_CLOAKED_APP, DWM_CLOAKED_INHERITED, DWM_CLOAKED_SHELL,
            },
            Gdi::{
                GetMonitorInfoW, MonitorFromWindow, HMONITOR, MONITORINFOEXW,
                MONITOR_DEFAULTTOPRIMARY,
            },
        },
        Storage::EnhancedStorage::PKEY_FileDescription,
        System::{
            Com::{CoCreateInstance, CLSCTX_ALL},
            RemoteDesktop::ProcessIdToSessionId,
            Threading::{
                GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW,
                PROCESS_ACCESS_RIGHTS, PROCESS_NAME_WIN32, PROCESS_QUERY_INFORMATION,
            },
        },
        UI::{
            Shell::{
                IShellItem2, IVirtualDesktopManager, SHCreateItemFromParsingName,
                VirtualDesktopManager, SIGDN_NORMALDISPLAY,
            },
            WindowsAndMessaging::{
                GetClassNameW, GetDesktopWindow, GetForegroundWindow, GetParent, GetWindowLongW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, IsZoomed, SetWindowPos, ShowWindow, ShowWindowAsync, SystemParametersInfoW, ANIMATIONINFO, GWL_EXSTYLE, GWL_STYLE, SET_WINDOW_POS_FLAGS, SHOW_WINDOW_CMD, SPIF_SENDCHANGE, SPI_GETANIMATION, SPI_SETANIMATION, SWP_ASYNCWINDOWPOS, SWP_NOZORDER, SW_MINIMIZE, SW_NORMAL, SW_RESTORE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WINDOW_EX_STYLE, WINDOW_STYLE
            },
        },
    },
};

use crate::error_handler::{log_if_error, Result};

pub struct WindowsApi {}
impl WindowsApi {
    pub fn window_thread_process_id(hwnd: HWND) -> (u32, u32) {
        let mut process_id: u32 = 0;

        // Behaviour is undefined if an invalid HWND is given
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid
        let thread_id = unsafe {
            GetWindowThreadProcessId(hwnd, Option::from(std::ptr::addr_of_mut!(process_id)))
        };

        (process_id, thread_id)
    }

    pub fn current_process_id() -> u32 {
        unsafe { GetCurrentProcessId() }
    }

    pub fn current_session_id() -> Result<u32> {
        let process_id = Self::current_process_id();
        let mut session_id = 0;

        unsafe {
            if ProcessIdToSessionId(process_id, &mut session_id).is_ok() {
                Ok(session_id)
            } else {
                Err(eyre!("could not determine current session id").into())
            }
        }
    }

    pub fn force_set_foregorund(hwnd: HWND) -> Result<()> {
        Self::set_minimize_animation(false)?;
        Self::show_window_async(hwnd, SW_MINIMIZE);
        Self::show_window_async(hwnd, SW_RESTORE);
        Self::set_minimize_animation(true)?;
        Ok(())
    }

    pub fn get_foreground_window() -> HWND {
        unsafe { GetForegroundWindow() }
    }

    pub fn is_window(hwnd: HWND) -> bool {
        unsafe { IsWindow(hwnd) }.into()
    }

    pub fn is_window_visible(hwnd: HWND) -> bool {
        unsafe { IsWindowVisible(hwnd) }.into()
    }

    pub fn is_iconic(hwnd: HWND) -> bool {
        unsafe { IsIconic(hwnd) }.into()
    }

    pub fn is_maximized(hwnd: HWND) -> bool {
        unsafe { IsZoomed(hwnd) }.into()
    }

    pub fn is_cloaked(hwnd: HWND) -> Result<bool> {
        let mut cloaked: u32 = 0;
        Self::dwm_get_window_attribute(hwnd, DWMWA_CLOAKED, &mut cloaked)?;
        Ok(matches!(
            cloaked,
            DWM_CLOAKED_APP | DWM_CLOAKED_SHELL | DWM_CLOAKED_INHERITED
        ))
    }

    pub fn show_window(hwnd: HWND, command: SHOW_WINDOW_CMD) {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
        unsafe { ShowWindow(hwnd, command) };
    }

    pub fn show_window_async(hwnd: HWND, command: SHOW_WINDOW_CMD) {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindowasync
        unsafe { ShowWindowAsync(hwnd, command) };
    }

    pub fn unmaximize_window(hwnd: HWND) {
        Self::show_window(hwnd, SW_NORMAL);
    }

    pub fn get_styles(hwnd: HWND) -> WINDOW_STYLE {
        WINDOW_STYLE(unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32)
    }

    pub fn get_ex_styles(hwnd: HWND) -> WINDOW_EX_STYLE {
        WINDOW_EX_STYLE(unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32)
    }

    fn _set_position(
        hwnd: HWND,
        order: HWND,
        rect: RECT,
        flags: SET_WINDOW_POS_FLAGS,
    ) -> Result<()> {
        unsafe {
            SetWindowPos(
                hwnd,
                order,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                flags,
            )?;
        }
        Ok(())
    }

    pub fn set_position(
        hwnd: HWND,
        order: Option<HWND>,
        rect: &RECT,
        flags: SET_WINDOW_POS_FLAGS,
    ) -> Result<()> {
        let uflags = match order {
            Some(_) => flags,
            None => SWP_NOZORDER | flags,
        };
        let order = order.unwrap_or(HWND(0));

        if uflags.contains(SWP_ASYNCWINDOWPOS) {
            let rect = rect.clone();
            std::thread::spawn(move || Self::_set_position(hwnd, order, rect, uflags));
            return Ok(());
        }

        Self::_set_position(hwnd, order, *rect, uflags)
    }

    fn open_process(
        access_rights: PROCESS_ACCESS_RIGHTS,
        inherit_handle: bool,
        process_id: u32,
    ) -> Result<HANDLE> {
        unsafe { Ok(OpenProcess(access_rights, inherit_handle, process_id)?) }
    }

    fn close_handle(handle: HANDLE) -> Result<()> {
        unsafe {
            CloseHandle(handle)?;
        }
        Ok(())
    }

    fn process_handle(process_id: u32) -> Result<HANDLE> {
        Self::open_process(PROCESS_QUERY_INFORMATION, false, process_id)
    }

    pub fn get_parent(hwnd: HWND) -> HWND {
        unsafe { GetParent(hwnd) }
    }

    pub fn exe_path(hwnd: HWND) -> Result<String> {
        let mut len = 512_u32;
        let mut path: Vec<u16> = vec![0; len as usize];
        let text_ptr = path.as_mut_ptr();

        let (process_id, _) = Self::window_thread_process_id(hwnd);
        let handle = Self::process_handle(process_id)?;
        unsafe {
            log_if_error(QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_WIN32,
                PWSTR(text_ptr),
                &mut len,
            ));
        }
        Self::close_handle(handle)?;

        Ok(String::from_utf16(&path[..len as usize])?)
    }

    pub fn exe(hwnd: HWND) -> Result<String> {
        Ok(Self::exe_path(hwnd)?
            .split('\\')
            .last()
            .ok_or_else(|| eyre!("there is no last element"))?
            .to_string())
    }

    pub fn get_class(hwnd: HWND) -> Result<String> {
        let mut text: [u16; 512] = [0; 512];
        let len = unsafe { GetClassNameW(hwnd, &mut text) };
        let length = usize::try_from(len).unwrap_or(0);
        Ok(String::from_utf16(&text[..length])?)
    }

    pub fn get_window_display_name(hwnd: HWND) -> Result<String> {
        let path = Self::exe_path(hwnd)?;
        let wide_path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        unsafe {
            let shell_item: IShellItem2 =
                SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None)?;
            match shell_item.GetString(&PKEY_FileDescription) {
                Ok(description) => Ok(description.to_string()?),
                Err(_) => Ok(shell_item
                    .GetDisplayName(SIGDN_NORMALDISPLAY)?
                    .to_string()?
                    .replace(".exe", "")),
            }
        }
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

    pub fn get_window_rect(hwnd: HWND) -> RECT {
        let mut rect = unsafe { std::mem::zeroed() };
        if Self::dwm_get_window_attribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &mut rect).is_ok() {
            rect
        } else {
            unsafe { GetWindowRect(hwnd, &mut rect).ok() };
            rect
        }
    }

    pub fn primary_monitor() -> HMONITOR {
        unsafe { MonitorFromWindow(GetDesktopWindow(), MONITOR_DEFAULTTOPRIMARY) }
    }

    pub fn monitor_info(hmonitor: HMONITOR) -> Result<MONITORINFOEXW> {
        let mut ex_info = MONITORINFOEXW::default();
        ex_info.monitorInfo.cbSize = u32::try_from(std::mem::size_of::<MONITORINFOEXW>())?;
        unsafe { GetMonitorInfoW(hmonitor, &mut ex_info.monitorInfo) };
        Ok(ex_info)
    }

    pub fn shadow_rect(hwnd: HWND) -> Result<RECT> {
        let window_rect = Self::get_window_rect(hwnd);

        let mut shadow_rect = Default::default();
        unsafe { GetWindowRect(hwnd, &mut shadow_rect)? };

        Ok(RECT {
            left: shadow_rect.left - window_rect.left,
            top: shadow_rect.top - window_rect.top,
            right: shadow_rect.right - window_rect.right,
            bottom: shadow_rect.bottom - window_rect.bottom,
        })
    }

    pub fn get_virtual_desktop_manager() -> Result<IVirtualDesktopManager> {
        unsafe {
            let manager: Result<IVirtualDesktopManager, windows::core::Error> =
                CoCreateInstance(&VirtualDesktopManager, None, CLSCTX_ALL);
            Ok(manager?)
        }
    }

    pub fn get_virtual_desktop_id(hwnd: HWND) -> Result<GUID> {
        let manager = Self::get_virtual_desktop_manager()?;
        let mut desktop_id = GUID::zeroed();
        let mut attempt = 0;
        while desktop_id.to_u128() == 0 && attempt < 10 {
            attempt += 1;
            sleep(Duration::from_millis(30));
            match unsafe { manager.GetWindowDesktopId(hwnd) } {
                Ok(desktop) => desktop_id = desktop,
                Err(_) => {}
            }
        }
        if desktop_id.to_u128() == 0 {
            return Err(eyre!("Failed to get desktop id for: {hwnd:?}").into());
        }
        Ok(desktop_id)
    }

    pub fn get_min_animation_info() -> Result<ANIMATIONINFO> {
        let mut anim_info: ANIMATIONINFO = unsafe { core::mem::zeroed() };
        anim_info.cbSize = core::mem::size_of::<ANIMATIONINFO>() as u32;
        let uiparam = anim_info.cbSize.clone();
        unsafe {
            SystemParametersInfoW(
                SPI_GETANIMATION,
                uiparam,
                Some(&mut anim_info as *mut ANIMATIONINFO as *mut c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            )?;
        }
        Ok(anim_info)
    }

    pub fn set_minimize_animation(enable: bool) -> Result<()> {
        let mut anim_info = Self::get_min_animation_info()?;
        let uiparam = anim_info.cbSize.clone();
        unsafe {
            anim_info.iMinAnimate = enable.into();
            SystemParametersInfoW(
                SPI_SETANIMATION,
                uiparam,
                Some(&mut anim_info as *mut ANIMATIONINFO as *mut c_void),
                SPIF_SENDCHANGE,
            )?;
        }
        Ok(())
    }
}

/*
may be this is useful later

static CHILD_FROM_FRAME: AtomicIsize = AtoumicIsize::new(0);
unsafe extern "system" fn enum_childs_uwp(hwnd: HWND, _: LPARAM) -> BOOL {
    let exe = WindowsApi::exe(hwnd).unwrap_or_default();
    println!("enum_childs_uwp {} {}", hwnd.0, exe);
    if exe != "ApplicationFrameHost.exe" {
        CHILD_FROM_FRAME.store(hwnd.0, Ordering::SeqCst);
        return false.into();
    }
    true.into()
}

pub fn get_child_from_frame_host(hwnd: HWND) -> HWND {
    CHILD_FROM_FRAME.store(0, Ordering::SeqCst);
    unsafe { EnumChildWindows(hwnd, Some(enum_childs_uwp), LPARAM(0)) };
    HWND(CHILD_FROM_FRAME.load(Ordering::SeqCst))
} */
