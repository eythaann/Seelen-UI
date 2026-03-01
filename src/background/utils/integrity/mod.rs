mod checksums;
mod webview;

pub use checksums::*;
pub use webview::*;

use itertools::Itertools;
use tauri::webview_version;
use windows::Win32::{
    Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
    System::Threading::CreateMutexW,
};

use crate::{
    error::Result,
    is_local_dev,
    utils::{has_fixed_runtime, is_running_as_appx, was_installed_using_msix},
    windows_api::{string_utils::WindowsString, WindowsApi},
};

use super::spawn_named_thread;

/// Prints information about the computer runtime context to help debugging.
#[rustfmt::skip]
pub fn print_initial_information() {
    let version = env!("CARGO_PKG_VERSION");
    let debug = if tauri::is_dev() { " (debug)" } else { "" };
    let local = if is_local_dev() { " (local)" } else { "" };

    let os = os_info::get();
    let sys_locale = seelen_core::state::Settings::get_locale();

    log::info!(
        "───────────────────── Starting Seelen UI v{version}{local}{debug} ─────────────────────"
    );

    log::info!("Arguments        : {:?}", std::env::args().collect_vec());
    log::info!("Working Directory: {:?}", std::env::current_dir());

    log::info!("Operating System : {}", os.os_type());
    log::info!("  version        : {}", os.version());
    log::info!("  edition        : {}", os.edition().unwrap_or("None"));
    log::info!("  codename       : {}", os.codename().unwrap_or("None"));
    log::info!("  bitness        : {}", os.bitness());
    log::info!("  architecture   : {}", os.architecture().unwrap_or("Unknown"));
    log::info!("  locate         : {}", sys_locale.unwrap_or("Unknown".to_owned()));

    /* let mut sys_info = sysinfo::System::new();
    sys_info.refresh_cpu();
    sys_info.refresh_memory();
    log::info!("Specs");
    log::info!("  CPU Threads    : {}", sys_info.cpus().len());
    log::info!("  Memory         : {}GB", sys_info.total_memory() / 1024 / 1024 / 1024); */

    log::info!("WebView2 Runtime : {:?}", webview_version());
    log::info!("  Fixed          : {:?}", has_fixed_runtime());

    log::info!("Build Profile    : {}", if cfg!(debug_assertions) { "debug" } else { "release" });
    log::info!("  Bundled with   : {}", if is_running_as_appx() { "APPX/MSIX" } else { "NSIS" });
}

pub fn restart_as_appx() -> Result<!> {
    WindowsApi::execute(
        r"shell:AppsFolder\Seelen.SeelenUI_p6yyn03m1894e!App".to_string(),
        None,
        None,
        false,
    )?;
    std::process::exit(0);
}

pub fn restart_as_interactive_user() -> Result<!> {
    let path = if was_installed_using_msix() {
        "shell:AppsFolder\\Seelen.SeelenUI_p6yyn03m1894e!App".to_string()
    } else {
        std::env::current_exe()?.to_string_lossy().to_string()
    };
    // Use explorer.exe to spawn the process de-elevated (with the interactive user's token).
    // ShellExecuteExW called from an elevated process inherits the elevated token,
    // causing an infinite restart loop when the service launches the app elevated.
    std::process::Command::new("explorer.exe")
        .arg(&path)
        .spawn()?;
    std::process::exit(0);
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
