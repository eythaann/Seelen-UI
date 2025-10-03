#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Windows: {0}")]
    Windows(#[from] windows::core::Error),
    #[error("Starting positioning failed")]
    StartingPositioningFailed,
    #[error("Positioning failed")]
    SetPositionFailed,
    #[error("Utf16: {0}")]
    Utf16(#[from] std::string::FromUtf16Error),
}

pub type Result<T> = core::result::Result<T, Error>;
