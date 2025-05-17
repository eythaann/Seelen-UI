use std::{
    fs,
    io::{BufRead, Write},
    net::TcpStream,
    path::PathBuf,
};

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use windows::Win32::{
    System::TaskScheduler::{IExecAction2, ITaskService, TaskScheduler},
    UI::Shell::FOLDERID_LocalAppData,
};
use windows_core::Interface;

use crate::{
    error_handler::Result,
    seelen::get_app_handle,
    utils::{pwsh::PwshScript, was_installed_using_msix},
    windows_api::{Com, WindowsApi},
};

use super::domain::{SvcAction, SvcMessage};

pub struct ServicePipe;
impl ServicePipe {
    fn token() -> &'static str {
        std::env!("SLU_SERVICE_CONNECTION_TOKEN")
    }

    fn socket_path() -> PathBuf {
        std::env::temp_dir().join("com.seelen.seelen-ui\\slu_service_tcp_socket")
    }

    fn connect_tcp() -> Result<TcpStream> {
        let port = fs::read_to_string(Self::socket_path())?;
        Ok(TcpStream::connect(format!("127.0.0.1:{port}"))?)
    }

    /// will ignore any response
    pub fn request(message: SvcAction) -> Result<()> {
        let stream = Self::connect_tcp()?;
        let mut writter = std::io::BufWriter::new(&stream);

        let data = serde_json::to_vec(&SvcMessage {
            token: Self::token().to_string(),
            action: message,
        })?;
        writter.write_all(&data)?;
        writter.write_all(&[0x17])?;
        writter.flush()?;
        Ok(())
    }

    /// will wait for a response
    #[allow(dead_code)]
    pub fn request_response(message: SvcAction) -> Result<Vec<u8>> {
        let stream = Self::connect_tcp()?;
        stream.set_read_timeout(Some(std::time::Duration::from_millis(1000)))?;

        let mut writter = std::io::BufWriter::new(&stream);
        let mut reader = std::io::BufReader::new(&stream);

        let data = serde_json::to_vec(&message)?;
        writter.write_all(&data)?;
        writter.write_all(&[0x17])?;
        writter.flush()?;

        let mut bytes = Vec::new();
        reader.read_until(0x17, &mut bytes)?; // Read until end of transmission block
        bytes.pop(); // Remove end of transmission block
        Ok(bytes)
    }

    pub fn is_running() -> bool {
        Self::connect_tcp().is_ok()
    }

    pub fn service_path() -> Result<PathBuf> {
        let service_path = if was_installed_using_msix() {
            WindowsApi::known_folder(FOLDERID_LocalAppData)?
                .join("Microsoft\\WindowsApps\\slu-service.exe")
        } else {
            std::env::current_exe()?.with_file_name("slu-service.exe")
        };
        Ok(service_path)
    }

    fn start_service_task() -> Result<()> {
        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(
                &Default::default(),
                &Default::default(),
                &Default::default(),
                &Default::default(),
            )?;
            let folder = task_service.GetFolder(&"\\Seelen".into())?;
            let task = folder.GetTask(&"Seelen UI Service".into())?;

            let actions = task.Definition()?.Actions()?;
            // ask to microsoft what that hell this start counting from 1 instead 0
            let action: IExecAction2 = actions.get_Item(1)?.cast()?;
            let mut action_path = windows_core::BSTR::new();
            action.Path(&mut action_path)?;

            let service_path = Self::service_path()?.to_string_lossy().to_lowercase();
            match action_path.to_string().to_lowercase() == service_path {
                true => {
                    task.Run(&Default::default())?;
                    Ok(())
                }
                false => {
                    Err("Service task is not pointing to the correct service executable".into())
                }
            }
        })
    }

    // the service should be running since installer will start it or startup task scheduler
    // so if the service is not running, we need to start it (common on msix setup)
    pub async fn start_service() -> Result<()> {
        if let Err(err) = Self::start_service_task() {
            log::debug!("Can not start service via task scheduler: {}", err);
            let script = PwshScript::new(format!(
                "Start-Process '{}' -Verb runAs",
                Self::service_path()?.display()
            ))
            .inline_command()
            .elevated();
            let result = script.execute().await;
            if let Err(err) = result {
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
                    script.execute().await?;
                }
                return Err(err);
            }
        }
        Ok(())
    }
}
