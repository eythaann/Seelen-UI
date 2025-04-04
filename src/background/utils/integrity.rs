use std::sync::atomic::{AtomicBool, Ordering};

use base64::Engine;
use itertools::Itertools;
use tauri::webview_version;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_shell::ShellExt;

use crate::{
    error_handler::Result, is_local_dev, utils::is_running_as_appx, windows_api::WindowsApi,
};

use super::spawn_named_thread;

pub fn register_panic_hook() {
    let base_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let cause = info
            .payload()
            .downcast_ref::<String>()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                info.payload()
                    .downcast_ref::<&str>()
                    .unwrap_or(&"<cause unknown>")
                    .to_string()
            });

        let mut string_location = String::from("<location unknown>");
        if let Some(location) = info.location() {
            string_location = format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }

        log::error!(
            "A panic occurred:\n  Cause: {}\n  Location: {}",
            cause,
            string_location
        );
        base_hook(info);
    }));
}

/// Prints information about the computer runtime context to help debugging.
pub fn print_initial_information() {
    let version = env!("CARGO_PKG_VERSION");
    let debug = if tauri::is_dev() { " (debug)" } else { "" };
    let local = if is_local_dev() { " (local)" } else { "" };
    let msix = if is_running_as_appx() { " (msix)" } else { "" };
    log::info!(
        "───────────────────── Starting Seelen UI v{version}{local}{debug}{msix} ─────────────────────"
    );
    let os = os_info::get();
    let sys_locale = seelen_core::state::Settings::get_locale();
    log::info!("Arguments       : {:?}", std::env::args().collect_vec());
    log::info!("Operating System: {}", os.os_type());
    log::info!("  version       : {}", os.version());
    log::info!("  edition       : {}", os.edition().unwrap_or("None"));
    log::info!("  codename      : {}", os.codename().unwrap_or("None"));
    log::info!("  bitness       : {}", os.bitness());
    log::info!(
        "  architecture  : {}",
        os.architecture().unwrap_or("Unknown")
    );
    log::info!(
        "  locate        : {}",
        sys_locale.unwrap_or("Unknown".to_owned())
    );
    log::info!("WebView2 Runtime: {:?}", webview_version());
    log::info!("Elevated        : {:?}", WindowsApi::is_elevated());
}

pub fn validate_webview_runtime_is_installed(app: &tauri::AppHandle) -> Result<()> {
    let error = match webview_version() {
        Ok(version) => {
            let mut version = version.split('.');
            let major = version.next().unwrap_or("0").parse().unwrap_or(0);
            if major < 110 {
                Some((
                    t!("runtime.outdated"),
                    t!("runtime.outdated_description", min_version = "110"),
                ))
            } else {
                None
            }
        }
        Err(_) => Some((t!("runtime.not_found"), t!("runtime.not_found_description"))),
    };

    if let Some((title, message)) = error {
        let ok_pressed = app
            .dialog()
            .message(message)
            .title(title)
            .kind(MessageDialogKind::Error)
            .buttons(MessageDialogButtons::OkCustom(
                t!("runtime.download").to_string(),
            ))
            .blocking_show();
        if ok_pressed {
            let url = "https://developer.microsoft.com/en-us/microsoft-edge/webview2/?form=MA13LH#download";
            #[allow(deprecated)]
            app.shell().open(url, None)?;
        }
        return Err("Webview runtime not installed or outdated".into());
    }
    Ok(())
}

static WEBVIEW_STATE_VALIDATED: AtomicBool = AtomicBool::new(false);

/// Try creating a webview window, tauri for some reason could panic stoping the setup hook and for some reason
/// the panic hook is not catching this so this implementation is a workaround for that.
///
/// The event loop is still running after fail so we can easily restart the app just using tauri.
pub fn check_for_webview_optimal_state(app: &tauri::AppHandle) -> Result<()> {
    start_integrity_thread(app.clone());
    let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("@seelen/integrity");
    let window = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::App("integrity/index.html".into()),
    )
    .visible(false)
    .focused(false)
    .build()?;
    window.hwnd()?; // build could not fail so we check for the handle.
    window.destroy()?; // close the fake window
    WEBVIEW_STATE_VALIDATED.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn start_integrity_thread(app: tauri::AppHandle) {
    spawn_named_thread("Integrity", move || {
        // Maximum number of attempts to validate the webview state (max: 5 seconds)
        let mut remaining_attempts = 50;
        while !WEBVIEW_STATE_VALIDATED.load(Ordering::SeqCst) && remaining_attempts > 0 {
            remaining_attempts -= 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        // If all attempts are exhausted, exit the application with an error code
        if remaining_attempts == 0 {
            app.exit(1);
        }
    })
    .expect("Failed to start integrity thread");
}

pub fn restart_as_appx() -> Result<!> {
    std::process::Command::new("C:\\Windows\\explorer.exe")
        .arg(r"shell:AppsFolder\Seelen.SeelenUI_p6yyn03m1894e!App")
        .spawn()?;
    std::process::exit(0);
}
