pub mod application;
pub mod domain;

use std::{
    fs,
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use application::{handle_cli_events, SEELEN_COMMAND_LINE};
use itertools::Itertools;

use crate::{
    error_handler::Result,
    log_error,
    seelen::Seelen,
    trace_lock,
    utils::{pwsh::PwshScript, spawn_named_thread},
};

pub struct AppClient;
impl AppClient {
    fn socket_path() -> PathBuf {
        std::env::temp_dir().join("com.seelen.seelen-ui\\slu_tcp_socket")
    }

    // const BUFFER_SIZE: usize = 5 * 1024 * 1024; // 5 MB
    fn handle_message(stream: TcpStream) {
        let argv = match serde_json::from_reader::<TcpStream, Vec<String>>(stream) {
            Ok(argv) => argv,
            Err(e) => {
                log::error!("Failed to deserialize message: {}", e);
                return;
            }
        };
        log::trace!(target: "slu::cli", "{}", argv[1..].join(" "));
        std::thread::spawn(move || {
            let command = trace_lock!(SEELEN_COMMAND_LINE).clone();
            if let Ok(matches) = command.try_get_matches_from(argv) {
                log_error!(handle_cli_events(&matches));
            }
        });
    }

    pub fn listen_tcp() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let socket_addr = listener.local_addr()?;
        let port = socket_addr.port();

        log::info!("TCP server listening on 127.0.0.1:{}", port);
        fs::write(Self::socket_path(), port.to_string())?;

        spawn_named_thread("TCP Listener", move || {
            for stream in listener.incoming() {
                if !Seelen::is_running() {
                    log::trace!("Exiting TCP Listener");
                    break;
                }
                match stream {
                    Ok(stream) => Self::handle_message(stream),
                    Err(e) => log::error!("Failed to accept connection: {}", e),
                }
            }
        })?;
        Ok(())
    }

    pub fn connect_tcp() -> Result<TcpStream> {
        let port = fs::read_to_string(Self::socket_path())?;
        Ok(TcpStream::connect(format!("127.0.0.1:{}", port))?)
    }

    pub fn redirect_cli_to_instance() -> Result<()> {
        let mut attempts: i32 = 0;
        let mut stream = Self::connect_tcp();
        while stream.is_err() && attempts < 10 {
            attempts += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
            stream = AppClient::connect_tcp();
        }
        serde_json::to_writer(stream?, &std::env::args().collect_vec())?;
        Ok(())
    }
}

pub struct ServiceClient;
impl ServiceClient {
    fn token() -> &'static str {
        std::option_env!("SLU_SERVICE_CONNECTION_TOKEN").unwrap_or("__local__")
    }

    fn socket_path() -> PathBuf {
        PathBuf::from(r"C:\Windows\Temp\slu_service_tcp_socket")
    }

    fn connect_tcp() -> Result<TcpStream> {
        let port = fs::read_to_string(Self::socket_path())?;
        Ok(TcpStream::connect(format!("127.0.0.1:{}", port))?)
    }

    fn send_message(message: &[&str]) -> Result<()> {
        let stream = Self::connect_tcp()?;
        let writter = std::io::BufWriter::new(stream);
        serde_json::to_writer(
            writter,
            &serde_json::json!({
                "token": Self::token(),
                "message": message,
            }),
        )?;
        Ok(())
    }

    pub fn is_running() -> bool {
        Self::connect_tcp().is_ok()
    }

    // the service should be running since installer will start it or startup task scheduler
    // so if the service is not running, we need to start it (common on msix setup)
    pub async fn start_service() -> Result<()> {
        /* get_app_handle()
            .dialog()
            .message(t!("service.not_running_description"))
            .title(t!("service.not_running"))
            .kind(MessageDialogKind::Info)
            .ok_button_label(t!("service.not_running_ok"))
            .blocking_show();
        */
        let service_path = std::env::current_exe()?.with_file_name("slu-service.exe");
        PwshScript::new(format!(
            "Start-Process '{}' -Verb runAs",
            service_path.display(),
        ))
        .inline_command()
        .execute()
        .await?;
        Ok(())
    }

    pub fn emit_stop_signal() -> Result<()> {
        Self::send_message(&["stop"])
    }

    pub fn emit_set_startup(enabled: bool) -> Result<()> {
        Self::send_message(&["set-startup", &enabled.to_string()])
    }
}
