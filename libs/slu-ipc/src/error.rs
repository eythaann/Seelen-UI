use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Service Error: {0}")]
    IpcResponseError(String),
    #[error("Serde Json Error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("IPC Timeout: {0}")]
    Timeout(String),
}

pub type Result<T = ()> = core::result::Result<T, Error>;
