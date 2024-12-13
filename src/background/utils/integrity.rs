use std::sync::atomic::{AtomicBool, Ordering};

use base64::Engine;
use tauri::webview_version;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_shell::ShellExt;

use crate::error_handler::Result;

use super::spawn_named_thread;

pub fn validate_webview_runtime_is_installed(app: &tauri::AppHandle) -> Result<()> {
    let mut title = "WebView2 Runtime not found";
    let mut message = "Seelen UI requires Webview2 Runtime. Please install it.";

    let major = match webview_version() {
        Ok(version) => {
            title = "WebView2 Runtime outdated";
            message = "Seelen UI requires Webview2 Runtime 110 or higher. Please update it.";
            let mut version = version.split('.');
            version.next().unwrap_or("0").parse().unwrap_or(0)
        }
        Err(_) => 0,
    };

    if major < 110 {
        let ok_pressed = app
            .dialog()
            .message(message)
            .title(title)
            .kind(MessageDialogKind::Error)
            .ok_button_label("Go to download page")
            .blocking_show();
        if ok_pressed {
            let url = "https://developer.microsoft.com/en-us/microsoft-edge/webview2/?form=MA13LH#download";
            #[allow(deprecated)]
            app.shell().open(url, None)?;
        }
        return Err(title.into());
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

pub fn start_slu_service(app: &mut tauri::App<tauri::Wry>) -> Result<()> {
    log::trace!("Starting slu-service");
    let path = std::env::current_exe()?;
    #[allow(deprecated)]
    app.shell().open(
        path.with_file_name("slu-service.exe")
            .to_string_lossy()
            .to_string(),
        None,
    )?;
    Ok(())
}

pub fn kill_slu_service() -> Result<()> {
    log::trace!("Killing slu-service");
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    let process = sys.processes().values().find(|p| {
        p.exe()
            .is_some_and(|path| path.ends_with("slu-service.exe"))
    });
    if let Some(process) = process {
        process.kill();
    }
    Ok(())
}
