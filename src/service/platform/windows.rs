use std::path::PathBuf;

use windows::Win32::{
    Foundation::{ERROR_ALREADY_EXISTS, GetLastError},
    Security::SE_TCB_NAME,
    System::Threading::CreateMutexW,
    UI::{Shell::FOLDERID_LocalAppData, WindowsAndMessaging::SW_MINIMIZE},
};

use crate::{
    error::Result,
    windows_api::{WindowsApi, string_utils::WindowsString},
};

pub fn prepare_process() -> Result<()> {
    WindowsApi::set_process_dpi_aware()?;
    WindowsApi::enable_privilege(SE_TCB_NAME)?;
    Ok(())
}

// https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw
pub fn is_already_running() -> bool {
    let session_id = WindowsApi::current_session_id();
    let mutex_name = format!("Local\\Seelen-UI-Service-Instance-{session_id}");
    let mutex_name_wide = WindowsString::from_str(&mutex_name);

    // Try to create a named mutex specific to the current session
    let Ok(handle) = (unsafe { CreateMutexW(None, true, mutex_name_wide.as_pcwstr()) }) else {
        // Failed to create mutex, assume not running to be safe
        log::warn!("Failed to create service instance mutex, proceeding anyway");
        return false;
    };

    // if mutex existed before, another instance is already running for this session
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return true;
    }

    // This is the first instance for this session
    // Keep the handle alive by leaking it (will be released when process exits)
    Box::leak(Box::new(handle));
    false
}

pub fn wait_for_native_shell() {
    WindowsApi::wait_for_native_shell();
}

pub fn minimize_console() {
    let window = WindowsApi::get_console_window();
    if !window.is_invalid() {
        let _ = WindowsApi::show_window(window.0 as _, SW_MINIMIZE.0);
    }
}

pub fn local_app_data() -> Result<PathBuf> {
    WindowsApi::known_folder(FOLDERID_LocalAppData)
}
