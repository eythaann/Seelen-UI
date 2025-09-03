use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error while encoding using bincode: {0}")]
    BincodeEncode(#[from] bincode::error::EncodeError),
    #[error("Error while decoding using bincode: {0}")]
    BincodeDecode(#[from] bincode::error::DecodeError),
    #[error("Service Error: {0}")]
    IpcResponseError(String),
}

pub type Result<T = ()> = core::result::Result<T, Error>;
