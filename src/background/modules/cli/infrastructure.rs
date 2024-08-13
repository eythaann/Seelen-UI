use std::{
    fs,
    io::{BufReader, Read},
    net::{TcpListener, TcpStream},
};

use crate::{
    error_handler::Result,
    log_error,
    modules::cli::application::{handle_cli_events, SEELEN_COMMAND_LINE},
    trace_lock,
};

pub struct Client;
impl Client {
    const BUFFER_SIZE: usize = 5 * 1024 * 1024; // 5 MB

    fn handle_message(stream: TcpStream) {
        let mut reader = BufReader::new(stream);
        let mut buffer = vec![];
        match reader.read_to_end(&mut buffer) {
            Ok(_) => {
                let message = String::from_utf8_lossy(&buffer).to_string();
                match serde_json::from_str::<Vec<String>>(&message) {
                    Ok(argv) => {
                        log::trace!(target: "slu::cli", "{}", argv[1..].join(" "));
                        std::thread::spawn(move || {
                            let command = trace_lock!(SEELEN_COMMAND_LINE).clone();
                            log_error!(handle_cli_events(&command.get_matches_from(argv)));
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to deserialize message: {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read from stream: {}", e);
            }
        }
    }

    pub fn listen_tcp() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let socket_addr = listener.local_addr()?;
        let port = socket_addr.port();

        log::info!("TCP server listening on 127.0.0.1:{}", port);
        fs::write(
            std::env::temp_dir().join("slu_tcp_socket"),
            port.to_string(),
        )?;

        std::thread::Builder::new()
            .stack_size(Self::BUFFER_SIZE)
            .spawn(move || {
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => Self::handle_message(stream),
                        Err(e) => {
                            log::error!("Failed to accept connection: {}", e);
                        }
                    }
                }
            })?;
        Ok(())
    }

    pub fn connect_tcp() -> Result<TcpStream> {
        let port = fs::read_to_string(std::env::temp_dir().join("slu_tcp_socket"))?;
        Ok(TcpStream::connect(format!("127.0.0.1:{}", port))?)
    }
}
