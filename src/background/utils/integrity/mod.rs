mod checksums;
mod webview;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

pub use checksums::*;
pub use webview::*;

use itertools::Itertools;
use tauri::webview_version;
use tauri_plugin_dialog::DialogExt;

use crate::{
    error::Result,
    is_local_dev,
    platform::{
        info::{has_fixed_runtime, is_running_as_appx},
        shell,
    },
};

#[derive(Debug)]
pub enum IntegrityError {
    WebviewRuntimeNotInstalled,
    WebviewRuntimeOutdated,
}

/// Prints information about the computer runtime context to help debugging.
#[rustfmt::skip]
pub fn print_initial_information() {
    let version = env!("CARGO_PKG_VERSION");
    let debug = if tauri::is_dev() { " (debug)" } else { "" };
    let local = if is_local_dev() { " (local)" } else { "" };

    let os = os_info::get();
    let sys_locale = seelen_core::state::Settings::get_system_locale();

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
    shell::open(r"shell:AppsFolder\Seelen.SeelenUI_p6yyn03m1894e!App")?;
    std::process::exit(0);
}

/// Warns the user when the app was explicitly started with administrator rights.
pub fn warn_if_elevated(app: &tauri::AppHandle) {
    if imp::is_elevated() {
        app.dialog()
            .message(t!("elevated.description"))
            .title(t!("elevated.title"))
            .kind(tauri_plugin_dialog::MessageDialogKind::Warning)
            .show(|_| {});
    }
}

/// Another Seelen UI instance is already running in the current OS user session.
pub fn is_already_running() -> bool {
    imp::is_already_running()
}
