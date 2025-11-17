use std::path::{Path, PathBuf};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HMODULE, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::WindowsAndMessaging::{SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, WH_CALLWNDPROC},
    },
};

use crate::{
    error::ResultLogExt, modules::system_tray::application::util::Util, windows_api::WindowsApi,
};

// ============================================================================
// DLL Function Types
// ============================================================================

/// Type for the GetMsgProc hook procedure exported by the DLL
type GetMsgProcFn = unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT;

// ============================================================================
// Hook Loader Implementation
// ============================================================================

// SAFETY: HHOOK is a Windows handle that is safe to share between threads
// even though it contains a raw pointer internally
unsafe impl Send for TrayHookLoader {}
unsafe impl Sync for TrayHookLoader {}

pub struct TrayHookLoader {
    _dll_handle: Option<HMODULE>,
    hook_handle: Option<HHOOK>,
}

impl TrayHookLoader {
    /// Creates a new loader, loads the DLL and installs the hook
    /// Events will be sent automatically through AppIpc
    pub fn new() -> crate::Result<Self> {
        let dll_path = Self::get_dll_path()?;
        let dll_handle = Self::load_dll(&dll_path)?;
        let call_msg_proc = Self::get_proc_address::<GetMsgProcFn>(dll_handle, "CallWndProc")?;

        let shell_tray = WindowsApi::find_window(None, None, None, Some("Shell_TrayWnd"))?;
        let (_pid, thread_id) = WindowsApi::window_thread_process_id(shell_tray);

        let hook_handle = unsafe {
            SetWindowsHookExW(
                WH_CALLWNDPROC,
                Some(call_msg_proc),
                Some(dll_handle.into()),
                thread_id, // 0 = global hook for all threads
            )
            .map_err(|e| format!("Failed to install hook: {:?}", e))?
        };

        log::info!("Tray hook DLL loaded and installed successfully");

        Util::refresh_icons().log_error();

        Ok(Self {
            _dll_handle: Some(dll_handle),
            hook_handle: Some(hook_handle),
        })
    }

    /// Gets the DLL path
    fn get_dll_path() -> crate::Result<PathBuf> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or("Failed to get executable directory")?;

        // The DLL should be in the same directory as the executable
        let dll_path = exe_dir.join("sluhk.dll");

        if !dll_path.exists() {
            return Err(format!("DLL not found at: {}", dll_path.display()).into());
        }

        Ok(dll_path)
    }

    /// Loads the DLL into memory
    fn load_dll(path: &Path) -> crate::Result<HMODULE> {
        let path_wide: Vec<u16> = path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let handle = LoadLibraryW(PCWSTR(path_wide.as_ptr()))?;
            Ok(handle)
        }
    }

    /// Gets the address of an exported function
    fn get_proc_address<F>(dll_handle: HMODULE, name: &str) -> crate::Result<F> {
        let name_cstr = std::ffi::CString::new(name)
            .map_err(|e| format!("Invalid function name '{}': {}", name, e))?;

        unsafe {
            let proc_addr =
                GetProcAddress(dll_handle, windows::core::PCSTR(name_cstr.as_ptr() as _));

            match proc_addr {
                Some(addr) => Ok(std::mem::transmute_copy(&addr)),
                None => Err(format!("Failed to get proc address for: {}", name).into()),
            }
        }
    }
}

impl Drop for TrayHookLoader {
    fn drop(&mut self) {
        // Uninstall the hook
        if let Some(hook) = self.hook_handle.take() {
            unsafe {
                if let Err(e) = UnhookWindowsHookEx(hook) {
                    log::warn!("Failed to uninstall hook: {:?}", e);
                }
            }
        }

        // The DLL will be freed automatically when the process terminates
        // No need to call FreeLibrary explicitly
        log::info!("Tray hook unloaded");
    }
}
