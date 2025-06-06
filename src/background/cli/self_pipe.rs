use std::{
    fs,
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use clap::Parser;

use crate::{
    cli::application::AppCli, error_handler::Result, log_error, seelen::Seelen,
    utils::spawn_named_thread,
};

pub struct SelfPipe;
impl SelfPipe {
    fn socket_path() -> Result<PathBuf> {
        let dir = std::env::temp_dir().join("com.seelen.seelen-ui");
        if !dir.exists() {
            fs::create_dir(&dir)?;
        }
        Ok(dir.join("slu_tcp_socket"))
    }

    fn handle_message(stream: TcpStream) -> Result<()> {
        let argv: Vec<String> = serde_json::from_reader(stream)?;
        if argv.is_empty() {
            return Ok(());
        }
        log::trace!(target: "slu::cli", "{}", argv[1..].join(" "));
        if let Ok(cli) = AppCli::try_parse_from(argv) {
            cli.process()?;
        }
        Ok(())
    }

    pub fn listen_tcp() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let socket_addr = listener.local_addr()?;
        let port = socket_addr.port();

        log::info!("TCP server listening on 127.0.0.1:{port}");
        fs::write(Self::socket_path()?, port.to_string())?;

        spawn_named_thread("TCP Listener", move || {
            // wait for app fully started before trying to handle messages
            while !Seelen::is_running() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            for stream in listener.incoming() {
                if !Seelen::is_running() {
                    log::trace!("Exiting TCP Listener");
                    break;
                }
                match stream {
                    Ok(stream) => {
                        std::thread::spawn(move || log_error!(Self::handle_message(stream)));
                    }
                    Err(e) => log::error!("Failed to accept connection: {e}"),
                }
            }
        })?;
        Ok(())
    }

    pub fn connect_tcp() -> Result<TcpStream> {
        let port = fs::read_to_string(Self::socket_path()?)?;
        Ok(TcpStream::connect(format!("127.0.0.1:{port}"))?)
    }

    pub fn request_open_settings() -> Result<()> {
        let stream = Self::connect_tcp()?;
        serde_json::to_writer(
            stream,
            &[
                std::env::current_exe()?.to_string_lossy().to_string(),
                "settings".to_owned(),
            ],
        )?;
        Ok(())
    }
}
