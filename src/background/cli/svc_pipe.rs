use std::path::PathBuf;

use slu_ipc::{IPC, ServiceIpc, messages::SvcAction};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::{
    app::get_app_handle,
    error::Result,
    get_tokio_handle,
    platform::{info::was_installed_using_msix, paths, shell, startup},
};

pub struct ServicePipe;

impl ServicePipe {
    /// will ignore any response
    pub fn request(message: SvcAction) -> Result<()> {
        get_tokio_handle().spawn(async move {
            if let Err(err) = ServiceIpc::send(message.clone()).await {
                log::error!("Error sending message to service {err}. Message: {message:?}");
            }
        });
        Ok(())
    }

    pub fn request_sync(message: SvcAction) -> Result<()> {
        ServiceIpc::send_blocking(message.clone())?.ok()?;
        Ok(())
    }

    pub fn is_running() -> bool {
        ServiceIpc::can_stablish_connection()
    }

    pub fn service_path() -> Result<PathBuf> {
        let service_path = if was_installed_using_msix() {
            paths::local_data_dir()?.join("Microsoft\\WindowsApps\\slu-service.exe")
        } else {
            std::env::current_exe()?.with_file_name("slu-service.exe")
        };
        Ok(service_path)
    }

    // the service should be running since installer will start it or startup task scheduler
    // so if the service is not running, we need to start it (common on msix setup)
    pub async fn start_service() -> Result<()> {
        let Err(err) = startup::launch_service(&Self::service_path()?) else {
            return Ok(());
        };

        log::debug!("Can not start service via startup entry: {err}");

        let service_path = Self::service_path()?;
        let result = shell::execute(&service_path.to_string_lossy(), None, None, true);

        if let Err(err) = result {
            log::error!("First attempt to start service failed: {err}");
            let try_again = get_app_handle()
                .dialog()
                .message(t!("service.not_running_description"))
                .title(t!("service.not_running"))
                .kind(MessageDialogKind::Info)
                .buttons(MessageDialogButtons::OkCustom(
                    t!("service.not_running_ok").to_string(),
                ))
                .blocking_show();

            if try_again {
                shell::execute(&service_path.to_string_lossy(), None, None, true)?;
            }
            return Err(err);
        }

        let mut counter = 0;
        while !Self::is_running() && counter < 10 {
            log::debug!("Waiting for service IPC...");
            counter += 1;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        if counter == 10 {
            get_app_handle()
                .dialog()
                .message(t!("service.not_running_description"))
                .title(t!("service.not_running"))
                .kind(MessageDialogKind::Error)
                .buttons(MessageDialogButtons::Ok)
                .blocking_show();
            return Err("Service not running".into());
        }

        Ok(())
    }
}
