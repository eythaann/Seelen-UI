use tauri::webview_version;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_shell::ShellExt;

use crate::error_handler::Result;

pub fn validate_webview_runtime_is_installed(app: &tauri::AppHandle) -> Result<()> {
    match webview_version() {
        Ok(_version) => Ok(()),
        Err(_) => {
            let ok_pressed = app
                .dialog()
                .message("Seelen UI requires Webview2 Runtime. Please install it.")
                .title("WebView2 Runtime not found")
                .kind(MessageDialogKind::Error)
                .ok_button_label("Go to download page")
                .blocking_show();
            if ok_pressed {
                let url = "https://developer.microsoft.com/en-us/microsoft-edge/webview2/?form=MA13LH#download";
                app.shell().open(url, None)?;
            }
            Err("Webview2 Runtime not found".into())
        }
    }
}

// will fail after 10 seconds if the webview is not in optimal state
pub fn wait_for_webview_optimal_state(app: &tauri::AppHandle) -> Result<()> {
    let attempts = 0;
    let check = || -> Result<()> {
        let window = tauri::WebviewWindowBuilder::new(
            app,
            "INTEGRITY",
            tauri::WebviewUrl::App("integrity/index.html".into()),
        )
        .visible(false)
        .build()?;
        window.hwnd()?; // build could not fail so we check for the handle.
        window.destroy()?; // close the fake window
        Ok(())
    };

    while check().is_err() && attempts < 10 {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    match attempts >= 10 {
        true => {
            app.dialog()
                .message("Please manually retry it after close this dialog.\nIf the problem persists, please contact the developer.")
                .title("Seelen UI failed to start")
                .kind(MessageDialogKind::Error)
                .blocking_show();
            Err("Failed to wait for webview optimal state".into())
        }
        false => Ok(()),
    }
}
