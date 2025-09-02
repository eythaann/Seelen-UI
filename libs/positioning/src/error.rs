#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Windows: {0}")]
    Windows(#[from] windows::core::Error),
    #[error("Starting positioning failed")]
    StartingPositioningFailed,
    #[error("Positioning failed")]
    SetPositionFailed,
}

pub type Result<T> = core::result::Result<T, Error>;
