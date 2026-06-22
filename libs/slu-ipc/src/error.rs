use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("IPC Response Error: {0}")]
    IpcResponse(String),
    #[error("Serde Json Error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("IPC Timeout: {0}")]
    Timeout(String),
    #[error("Windows Error: {0}")]
    Windows(#[from] windows::core::Error),
}

pub type Result<T = ()> = core::result::Result<T, Error>;
