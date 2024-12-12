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
        let mut attempts = 0;
        while !WEBVIEW_STATE_VALIDATED.load(Ordering::SeqCst) && attempts < 5 {
            attempts += 1;
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
        if !WEBVIEW_STATE_VALIDATED.load(Ordering::SeqCst) {
            app.exit(1);
        }
    })
    .expect("Failed to start integrity thread");
}
