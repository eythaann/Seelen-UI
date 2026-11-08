use windows::Win32::{
    Foundation::{ERROR_ALREADY_EXISTS, GetLastError},
    System::Threading::CreateMutexW,
};
use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

use crate::windows_api::{WindowsApi, string_utils::WindowsString};

/// Elevated while UAC is enabled: the app was explicitly started as administrator.
pub fn is_elevated() -> bool {
    is_uac_enabled() && WindowsApi::is_elevated().unwrap_or(false)
}

fn is_uac_enabled() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(key) = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System")
    else {
        return true; // assume enabled if we can't read
    };
    let enable_lua: u32 = key.get_value("EnableLUA").unwrap_or(1);
    enable_lua != 0
}

// https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw
pub fn is_already_running() -> bool {
    unsafe {
        let session_id = WindowsApi::current_session_id();
        let mutex_name = format!("Local\\Seelen-UI-Instance-{}", session_id);
        let mutex_name_wide = WindowsString::from_str(&mutex_name);

        // Try to create a named mutex specific to the current session
        let Ok(handle) = CreateMutexW(None, true, mutex_name_wide.as_pcwstr()) else {
            // Failed to create mutex, assume not running to be safe
            log::warn!("Failed to create instance mutex, proceeding anyway");
            return false;
        };

        // if mutex existed before, another instance is already running for this session
        let last_error = GetLastError();
        if last_error == ERROR_ALREADY_EXISTS {
            return true;
        }

        // This is the first instance for this session
        // Keep the handle alive by leaking it (will be released when process exits)
        Box::leak(Box::new(handle));
        false
    }
}
