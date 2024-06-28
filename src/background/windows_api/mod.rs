mod app_bar;
mod com;

pub use app_bar::*;
pub use com::*;

use std::{ffi::c_void, thread::sleep, time::Duration};

use color_eyre::eyre::eyre;
use windows::{
    core::{GUID, PCWSTR, PWSTR},
    Win32::{
        Devices::Display::{
            GetNumberOfPhysicalMonitorsFromHMONITOR, GetPhysicalMonitorsFromHMONITOR,
            PHYSICAL_MONITOR,
        },
        Foundation::{CloseHandle, HANDLE, HMODULE, HWND, LPARAM, LUID, RECT},
        Graphics::{
            Dwm::{
                DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
                DWMWINDOWATTRIBUTE, DWM_CLOAKED_APP, DWM_CLOAKED_INHERITED, DWM_CLOAKED_SHELL,
            },
            Gdi::{
                EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, HDC, HMONITOR,
                MONITORENUMPROC, MONITORINFOEXW, MONITOR_DEFAULTTOPRIMARY,
            },
        },
        Media::Audio::{
            eMultimedia, eRender, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator,
            MMDeviceEnumerator,
        },
        Security::{
            GetTokenInformation, LookupPrivilegeValueW, TokenElevation, TOKEN_ADJUST_PRIVILEGES,
            TOKEN_ELEVATION, TOKEN_QUERY,
        },
        Storage::EnhancedStorage::PKEY_FileDescription,
        System::{
            Com::CLSCTX_ALL,
            LibraryLoader::GetModuleHandleW,
            Power::{GetSystemPowerStatus, SetSuspendState, SYSTEM_POWER_STATUS},
            RemoteDesktop::ProcessIdToSessionId,
            Shutdown::{ExitWindowsEx, EXIT_WINDOWS_FLAGS, SHUTDOWN_REASON},
            Threading::{
                GetCurrentProcess, GetCurrentProcessId, OpenProcess, OpenProcessToken,
                QueryFullProcessImageNameW, PROCESS_ACCESS_RIGHTS, PROCESS_NAME_WIN32,
                PROCESS_QUERY_INFORMATION,
            },
        },
        UI::{
            HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
            Shell::{
                IShellItem2, IVirtualDesktopManager, SHCreateItemFromParsingName,
                VirtualDesktopManager, SIGDN_NORMALDISPLAY,
            },
            WindowsAndMessaging::{
                EnumWindows, GetClassNameW, GetDesktopWindow, GetForegroundWindow, GetParent,
                GetWindowLongW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
                IsWindow, IsWindowVisible, IsZoomed, SetWindowPos, ShowWindow, ShowWindowAsync,
                SystemParametersInfoW, ANIMATIONINFO, EVENT_SYSTEM_FOREGROUND, GWL_EXSTYLE,
                GWL_STYLE, SET_WINDOW_POS_FLAGS, SHOW_WINDOW_CMD, SPIF_SENDCHANGE,
                SPI_GETANIMATION, SPI_SETANIMATION, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE,
                SWP_NOSIZE, SWP_NOZORDER, SW_MINIMIZE, SW_NORMAL, SW_RESTORE,
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WINDOW_EX_STYLE, WINDOW_STYLE, WNDENUMPROC,
            },
        },
    },
};

use crate::{error_handler::Result, hook::HOOK_MANAGER, log_error, winevent::WinEvent};

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
        unsafe { EnumDisplayMonitors(HDC(0), None, callback, LPARAM(callback_data_address)) }
            .ok()?;
        Ok(())
    }

    pub fn enum_windows(callback: WNDENUMPROC, callback_data_address: isize) -> Result<()> {
        unsafe { EnumWindows(callback, LPARAM(callback_data_address))? };
        Ok(())
    }

    pub fn get_device_pixel_ratio(hmonitor: HMONITOR) -> Result<f32> {
        let mut dpi_x: u32 = 0;
        let mut _dpi_y: u32 = 0;
        unsafe { GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut _dpi_y)? };
        // 96 is the default DPI value on Windows
        Ok(dpi_x as f32 / 96_f32)
    }

    pub fn window_thread_process_id(hwnd: HWND) -> (u32, u32) {
        let mut process_id: u32 = 0;

        // Behaviour is undefined if an invalid HWND is given
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid
        let thread_id = unsafe {
            GetWindowThreadProcessId(hwnd, Option::from(std::ptr::addr_of_mut!(process_id)))
        };

        (process_id, thread_id)
    }

    pub fn current_process() -> HANDLE {
        unsafe { GetCurrentProcess() }
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

    pub fn is_fullscreen(hwnd: HWND) -> Result<bool> {
        let rc_monitor = WindowsApi::monitor_rect(WindowsApi::monitor_from_window(hwnd))?;
        let window_rect = WindowsApi::get_window_rect(hwnd);
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

    pub fn show_window(hwnd: HWND, command: SHOW_WINDOW_CMD) -> Result<()> {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
        let result = unsafe { ShowWindow(hwnd, command) }.ok();
        if let Err(error) = result {
            if !error.code().is_ok() {
                return Err(error.into());
            }
        }
        Ok(())
    }

    pub fn show_window_async(hwnd: HWND, command: SHOW_WINDOW_CMD) -> Result<()> {
        // BOOL is returned but does not signify whether or not the operation was succesful
        // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindowasync
        let result = unsafe { ShowWindowAsync(hwnd, command) }.ok();
        if let Err(error) = result {
            if !error.code().is_ok() {
                return Err(error.into());
            }
        }
        Ok(())
    }

    pub fn unmaximize_window(hwnd: HWND) -> Result<()> {
        Self::show_window(hwnd, SW_NORMAL)
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
                (rect.right - rect.left).abs(),
                (rect.bottom - rect.top).abs(),
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
            let rect = *rect;
            std::thread::spawn(move || Self::_set_position(hwnd, order, rect, uflags));
            return Ok(());
        }

        Self::_set_position(hwnd, order, *rect, uflags)
    }

    pub fn move_window(hwnd: HWND, rect: &RECT) -> Result<()> {
        Self::set_position(hwnd, None, rect, SWP_NOSIZE | SWP_NOACTIVATE)
    }

    pub fn bring_to(hwnd: HWND, after: HWND) -> Result<()> {
        Self::set_position(
            hwnd,
            Some(after),
            &Default::default(),
            SWP_ASYNCWINDOWPOS | SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )?;
        Ok(())
    }

    pub fn force_set_foreground(hwnd: HWND) -> Result<()> {
        Self::set_minimize_animation(false)?;

        let mut hook_manager = HOOK_MANAGER.lock();
        hook_manager.pause_and_resume_after(WinEvent::SystemMinimizeEnd, hwnd);
        hook_manager.set_resume_callback(move |hook_manager| {
            log_error!(Self::set_minimize_animation(true));
            hook_manager.emit_fake_win_event(EVENT_SYSTEM_FOREGROUND, hwnd);
        });

        Self::show_window_async(hwnd, SW_MINIMIZE)?;
        Self::show_window_async(hwnd, SW_RESTORE)?;
        Ok(())
    }

    fn open_process(
        access_rights: PROCESS_ACCESS_RIGHTS,
        inherit_handle: bool,
        process_id: u32,
    ) -> Result<HANDLE> {
        unsafe { Ok(OpenProcess(access_rights, inherit_handle, process_id)?) }
    }

    pub fn open_process_token() -> Result<HANDLE> {
        let mut token_handle: HANDLE = HANDLE(0);
        unsafe {
            OpenProcessToken(
                Self::current_process(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token_handle,
            )?;
        }
        Ok(token_handle)
    }

    pub fn get_luid(system: PCWSTR, name: PCWSTR) -> Result<LUID> {
        let mut luid = LUID::default();
        unsafe { LookupPrivilegeValueW(system, name, &mut luid)? };
        Ok(luid)
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
            log_error!(QueryFullProcessImageNameW(
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
        unsafe { GetWindowRect(hwnd, &mut rect).ok() };
        rect
    }

    // some windows like explorer.exe have a shadow margin
    pub fn get_window_rect_without_margins(hwnd: HWND) -> RECT {
        let mut rect = unsafe { std::mem::zeroed() };
        if Self::dwm_get_window_attribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &mut rect).is_ok() {
            rect
        } else {
            unsafe { GetWindowRect(hwnd, &mut rect).ok() };
            rect
        }
    }

    pub fn monitor_from_window(hwnd: HWND) -> HMONITOR {
        unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) }
    }

    pub fn primary_monitor() -> HMONITOR {
        unsafe { MonitorFromWindow(GetDesktopWindow(), MONITOR_DEFAULTTOPRIMARY) }
    }

    /// handle of PHYSICAL_MONITOR is bugged and will be always 0
    pub fn primary_physical_monitor() -> Result<PHYSICAL_MONITOR> {
        let hmonitor = Self::primary_monitor();

        let mut c_physical_monitors: u32 = 0;
        let mut p_physical_monitors: Vec<PHYSICAL_MONITOR> = Vec::new();

        unsafe {
            GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut c_physical_monitors)?;
            p_physical_monitors.resize(c_physical_monitors as usize, std::mem::zeroed());
            GetPhysicalMonitorsFromHMONITOR(hmonitor, p_physical_monitors.as_mut())?;
        };

        Ok(p_physical_monitors[0])
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
        let window_rect = Self::get_window_rect_without_margins(hwnd);

        let mut shadow_rect = Default::default();
        unsafe { GetWindowRect(hwnd, &mut shadow_rect)? };

        Ok(RECT {
            left: shadow_rect.left - window_rect.left,
            top: shadow_rect.top - window_rect.top,
            right: shadow_rect.right - window_rect.right,
            bottom: shadow_rect.bottom - window_rect.bottom,
        })
    }

    pub fn _get_virtual_desktop_manager() -> Result<IVirtualDesktopManager> {
        Com::create_instance(&VirtualDesktopManager)
    }

    pub fn get_media_device_enumerator() -> Result<IMMDeviceEnumerator> {
        Com::create_instance::<IMMDeviceEnumerator>(&MMDeviceEnumerator)
    }

    pub fn get_default_audio_endpoint() -> Result<IAudioEndpointVolume> {
        let enumerator = Self::get_media_device_enumerator()?;
        let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)? };
        let endpoint: IAudioEndpointVolume = unsafe { device.Activate(CLSCTX_ALL, None)? };
        Ok(endpoint)
    }

    pub fn _get_virtual_desktop_id(hwnd: HWND) -> Result<GUID> {
        let manager = Self::_get_virtual_desktop_manager()?;
        let mut desktop_id = GUID::zeroed();
        let mut attempt = 0;
        while desktop_id.to_u128() == 0 && attempt < 10 {
            attempt += 1;
            sleep(Duration::from_millis(30));
            if let Ok(desktop) = unsafe { manager.GetWindowDesktopId(hwnd) } { desktop_id = desktop }
        }
        if desktop_id.to_u128() == 0 {
            return Err(eyre!("Failed to get desktop id for: {hwnd:?}").into());
        }
        Ok(desktop_id)
    }

    pub fn get_min_animation_info() -> Result<ANIMATIONINFO> {
        let mut anim_info: ANIMATIONINFO = unsafe { core::mem::zeroed() };
        anim_info.cbSize = core::mem::size_of::<ANIMATIONINFO>() as u32;
        let uiparam = anim_info.cbSize;
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
        let uiparam = anim_info.cbSize;
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

    pub fn exit_windows(flags: EXIT_WINDOWS_FLAGS, reason: SHUTDOWN_REASON) -> Result<()> {
        unsafe { ExitWindowsEx(flags, reason) }?;
        Ok(())
    }

    pub fn set_suspend_state() -> Result<()> {
        let success = unsafe { SetSuspendState(false, true, false).as_bool() };
        if !success {
            return Err(eyre!("Failed to set suspend state").into());
        }
        Ok(())
    }

    pub fn is_elevated() -> Result<bool> {
        unsafe {
            let mut elevation = TOKEN_ELEVATION::default();
            let mut ret_len = 0;

            let token_handle = Self::open_process_token()?;

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
