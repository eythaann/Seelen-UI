pub mod application;
pub mod domain;

use std::{
    fs,
    io::{BufRead, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use application::{get_app_command, handle_cli_events};
pub use domain::SvcAction;
use domain::SvcMessage;
use itertools::Itertools;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use windows::Win32::{
    System::TaskScheduler::{IExecAction2, ITaskService, TaskScheduler},
    UI::Shell::FOLDERID_LocalAppData,
};
use windows_core::Interface;

use crate::{
    error_handler::Result,
    log_error,
    seelen::{get_app_handle, Seelen},
    utils::{pwsh::PwshScript, spawn_named_thread, was_installed_using_msix},
    windows_api::{Com, WindowsApi},
};

pub struct AppClient;
impl AppClient {
    fn socket_path() -> Result<PathBuf> {
        let dir = std::env::temp_dir().join("com.seelen.seelen-ui");
        if !dir.exists() {
            fs::create_dir(&dir)?;
        }
        Ok(dir.join("slu_tcp_socket"))
    }

    // const BUFFER_SIZE: usize = 5 * 1024 * 1024; // 5 MB
    fn handle_message(stream: TcpStream) -> Result<()> {
        let argv: Vec<String> = serde_json::from_reader(stream)?;
        log::trace!(target: "slu::cli", "{}", argv[1..].join(" "));
        if let Ok(matches) = get_app_command().try_get_matches_from(argv) {
            handle_cli_events(&matches)?;
        }
        Ok(())
    }

    pub fn listen_tcp() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let socket_addr = listener.local_addr()?;
        let port = socket_addr.port();

        log::info!("TCP server listening on 127.0.0.1:{}", port);
        fs::write(Self::socket_path()?, port.to_string())?;

        spawn_named_thread("TCP Listener", move || {
            for stream in listener.incoming() {
                if !Seelen::is_running() {
                    log::trace!("Exiting TCP Listener");
                    break;
                }
                match stream {
                    Ok(stream) => {
                        std::thread::spawn(move || log_error!(Self::handle_message(stream)));
                    }
                    Err(e) => log::error!("Failed to accept connection: {}", e),
                }
            }
        })?;
        Ok(())
    }

    pub fn connect_tcp() -> Result<TcpStream> {
        let port = fs::read_to_string(Self::socket_path()?)?;
        Ok(TcpStream::connect(format!("127.0.0.1:{}", port))?)
    }

    pub fn open_settings() -> Result<()> {
        let stream = Self::connect_tcp()?;
        serde_json::to_writer(stream, &["settings"])?;
        Ok(())
    }

    /// will fail if no instance is running
    pub fn redirect_cli_to_instance() -> Result<()> {
        let stream = Self::connect_tcp()?;
        serde_json::to_writer(stream, &std::env::args().collect_vec())?;
        Ok(())
    }
}

pub struct ServiceClient;
impl ServiceClient {
    fn token() -> &'static str {
        std::env!("SLU_SERVICE_CONNECTION_TOKEN")
    }

    fn socket_path() -> PathBuf {
        std::env::temp_dir().join("slu_service_tcp_socket")
    }

    fn connect_tcp() -> Result<TcpStream> {
        let port = fs::read_to_string(Self::socket_path())?;
        Ok(TcpStream::connect(format!("127.0.0.1:{}", port))?)
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
            task_service.Connect(None, None, None, None)?;
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
                    task.Run(None)?;
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
