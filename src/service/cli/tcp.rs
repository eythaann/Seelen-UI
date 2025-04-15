use std::{
    io::{BufRead, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use crate::{
    cli::actions::SvcMessage, error::Result, task_scheduler::TaskSchedulerHelper,
    windows_api::WindowsApi,
};

use super::actions::SvcAction;

pub fn process_action(command: SvcAction) -> Result<String> {
    match command {
        SvcAction::Stop => crate::stop(),
        SvcAction::SetStartup(enabled) => TaskSchedulerHelper::set_run_on_logon(enabled)?,
        SvcAction::ShowWindow { hwnd, command } => WindowsApi::show_window(hwnd, command)?,
        SvcAction::ShowWindowAsync { hwnd, command } => {
            WindowsApi::show_window_async(hwnd, command)?
        }
        SvcAction::SetWindowPosition {
            hwnd,
            x,
            y,
            width,
            height,
            flags,
        } => WindowsApi::set_position(hwnd, x, y, width, height, flags)?,
        SvcAction::SetForeground(hwnd) => WindowsApi::set_foreground(hwnd)?,
    }
    Ok(String::new())
}

pub struct TcpService;
impl TcpService {
    fn token() -> &'static str {
        std::env!("SLU_SERVICE_CONNECTION_TOKEN")
    }

    fn socket_path() -> Result<PathBuf> {
        let dir = std::env::temp_dir().join("com.seelen.seelen-ui");
        if !dir.exists() {
            std::fs::create_dir(&dir)?;
        }
        Ok(dir.join("slu_service_tcp_socket"))
    }

    fn handle_message(stream: TcpStream) -> Result<()> {
        stream.set_read_timeout(Some(std::time::Duration::from_millis(1000)))?;

        let mut bytes = Vec::new();
        let mut reader = std::io::BufReader::new(&stream);
        reader.read_until(0x17, &mut bytes)?; // Read until end of transmission block
        bytes.pop(); // Remove end of transmission block
        let message: SvcMessage = serde_json::from_slice(&bytes)?;
        log::trace!("TCP command received: {:?}", message.action);

        if message.token != Self::token() {
            log::warn!("Unauthorized connection");
            return Ok(());
        }

        let result = process_action(message.action);
        let mut writter = std::io::BufWriter::new(&stream);
        if let Ok(res) = &result {
            writter.write_all(res.as_bytes())?;
        }
        writter.write_all(&[0x17])?;
        writter.flush()?;
        result?;
        Ok(())
    }

    pub fn listen_tcp() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let socket_addr = listener.local_addr()?;
        let port = socket_addr.port();

        log::info!("TCP server listening on 127.0.0.1:{}", port);
        std::fs::write(Self::socket_path()?, port.to_string())?;

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        if let Err(e) = Self::handle_message(stream) {
                            log::error!("Failed to handle message: {}", e);
                        }
                    }
                    Err(e) => log::error!("Failed to accept connection: {}", e),
                }
            }
        });
        Ok(())
    }

    pub fn connect_tcp() -> Result<TcpStream> {
        let port = std::fs::read_to_string(Self::socket_path()?)?;
        Ok(TcpStream::connect(format!("127.0.0.1:{}", port))?)
    }

    fn send(message: SvcAction) -> Result<()> {
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

    pub fn emit_stop_signal() -> Result<()> {
        Self::send(SvcAction::Stop)
    }
}

pub struct TcpBgApp;
impl TcpBgApp {
    fn socket_path() -> PathBuf {
        std::env::temp_dir().join("com.seelen.seelen-ui\\slu_tcp_socket")
    }

    pub fn connect_tcp() -> Result<TcpStream> {
        let port = std::fs::read_to_string(Self::socket_path())?;
        Ok(TcpStream::connect(format!("127.0.0.1:{}", port))?)
    }

    pub fn is_running() -> bool {
        if let Ok(stream) = Self::connect_tcp() {
            return serde_json::to_writer(stream, &serde_json::json!([])).is_ok();
        }
        false
    }
}
