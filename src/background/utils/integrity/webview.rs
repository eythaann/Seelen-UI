use base64::Engine;
use tauri::webview_version;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_shell::ShellExt;

use crate::{app::get_app_handle, error::Result, widgets::webview::WebviewArgs};

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

/// Try creating a webview window, tauri for some reason could panic stoping the setup hook and for some reason
/// the panic hook is not catching this so this implementation is a workaround for that.
pub async fn check_for_webview_optimal_state() -> Result<()> {
    log::info!("Testing webview optimal state...");

    let (tx, rx) = tokio::sync::oneshot::channel();

    std::thread::spawn(|| {
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("@seelen/integrity");
        let args = WebviewArgs::default();

        let window = tauri::WebviewWindowBuilder::new(
            get_app_handle(),
            &label,
            tauri::WebviewUrl::App("vanilla/integrity/index.html".into()),
        )
        .visible(false)
        .data_directory(args.data_directory())
        .additional_browser_args(&args.to_string())
        .build()?;
        window.hwnd()?; // build could not fail so we check for the handle.
        window.destroy()?; // close the fake window
        let _ = tx.send(());
        Result::Ok(())
    });

    tokio::select! {
        _ = rx => {
            log::info!("Webview optimal state confirmed.");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
            log::error!("Webview optimal state check timed out.");
            return Err("Webview optimal state check timed out".into());
        }
    }

    Ok(())
}
