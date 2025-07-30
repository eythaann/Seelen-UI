use std::{net::TcpStream, path::PathBuf};

use crate::error::Result;

pub struct TcpBgApp;
impl TcpBgApp {
    fn socket_path() -> PathBuf {
        std::env::temp_dir().join("com.seelen.seelen-ui\\slu_tcp_socket")
    }

    pub fn connect_tcp() -> Result<TcpStream> {
        let port = std::fs::read_to_string(Self::socket_path())?;
        Ok(TcpStream::connect(format!("127.0.0.1:{port}"))?)
    }

    pub fn is_running() -> bool {
        if let Ok(stream) = Self::connect_tcp() {
            return serde_json::to_writer(stream, &serde_json::json!([])).is_ok();
        }
        false
    }
}
