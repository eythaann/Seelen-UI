use color_eyre::eyre::eyre;
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HANDLE, HWND, LPARAM, RECT, BOOL},
        Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS, DWMWINDOWATTRIBUTE},
        System::{
            Com::{CoCreateInstance, CLSCTX_ALL},
            StationsAndDesktops::EnumDesktopsW,
            Threading::{
                AttachThreadInput, GetCurrentProcessId, GetCurrentThreadId, OpenProcess,
                QueryFullProcessImageNameW, PROCESS_ACCESS_RIGHTS, PROCESS_NAME_WIN32,
                PROCESS_QUERY_INFORMATION,
            },
        },
        UI::{
            Input::KeyboardAndMouse::SetFocus,
            Shell::{IVirtualDesktopManager, IVirtualDesktopManager_Vtbl, VirtualDesktopManager},
            WindowsAndMessaging::{
                AllowSetForegroundWindow, GetParent, GetWindowRect, GetWindowTextW,
                GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, SetForegroundWindow,
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

    pub fn current_thread_id() -> u32 {
        unsafe { GetCurrentThreadId() }
    }

    pub fn attach_thread_input(thread_id: u32, target_thread_id: u32, attach: bool) -> Result<()> {
        unsafe {
            AttachThreadInput(thread_id, target_thread_id, attach);
        }
        Ok(())
    }

    pub fn allow_set_foreground_window(process_id: u32) -> Result<()> {
        unsafe {
            AllowSetForegroundWindow(process_id)?;
        }
        Ok(())
    }

    pub fn set_foreground_window(hwnd: HWND) -> Result<()> {
        unsafe {
            SetForegroundWindow(hwnd);
        }
        Ok(())
    }

    pub fn set_focus(hwnd: HWND) -> Result<()> {
        unsafe { SetFocus(hwnd) };
        Ok(())
    }

    pub fn current_process_id() -> u32 {
        unsafe { GetCurrentProcessId() }
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

    fn open_process(
        access_rights: PROCESS_ACCESS_RIGHTS,
        inherit_handle: bool,
        process_id: u32,
    ) -> Result<HANDLE> {
        unsafe { Ok(OpenProcess(access_rights, inherit_handle, process_id)?) }
    }

    pub fn close_process(handle: HANDLE) -> Result<()> {
        unsafe {
            CloseHandle(handle)?;
        }
        Ok(())
    }

    pub fn process_handle(process_id: u32) -> Result<HANDLE> {
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
        Self::close_process(handle)?;

        Ok(String::from_utf16(&path[..len as usize])?)
    }

    pub fn exe(hwnd: HWND) -> Result<String> {
        Ok(Self::exe_path(hwnd)?
            .split('\\')
            .last()
            .ok_or_else(|| eyre!("there is no last element"))?
            .to_string())
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
                u32::try_from(std::mem::size_of::<T>()).unwrap(),
            );
        }

        Ok(())
    }

    pub fn get_window_rect(hwnd: HWND) -> RECT {
        let mut rect = unsafe { std::mem::zeroed() };
        if Self::dwm_get_window_attribute(hwnd, DWMWA_EXTENDED_FRAME_BOUNDS, &mut rect).is_ok() {
            rect
        } else {
            unsafe { GetWindowRect(hwnd, &mut rect) };
            rect
        }
    }

    pub fn shadow_rect(hwnd: HWND) -> Result<RECT> {
        let window_rect = Self::get_window_rect(hwnd);

        let mut shadow_rect = Default::default();
        unsafe { GetWindowRect(hwnd, &mut shadow_rect) };

        Ok(RECT {
            left: shadow_rect.left - window_rect.left,
            top: shadow_rect.top - window_rect.top,
            right: shadow_rect.right - window_rect.right,
            bottom: shadow_rect.bottom - window_rect.bottom,
        })
    }

    pub fn get_virtual_desktop_manager() -> Result<IVirtualDesktopManager> {
        unsafe {
            let manager: Result<IVirtualDesktopManager, windows::core::Error> = CoCreateInstance(
                &VirtualDesktopManager as *const _ as *const _,
                None,
                CLSCTX_ALL,
            );
            Ok(manager?)
        }
    }
}


unsafe extern "system" fn enum_desktops_proc(hwnd: PCWSTR, _: LPARAM) -> BOOL {
    println!("enum_desktops_proc {:?}", hwnd);
    true.into()
}
pub fn switch_desktop(idx: u32) {
    unsafe {
        EnumDesktopsW(None, Some(enum_desktops_proc), LPARAM(0));
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
