use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Tauri(tauri::Error),
    TauriCli(tauri_plugin_cli::Error),
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Io(value)
    }
}

impl From<tauri::Error> for AppError {
    fn from(value: tauri::Error) -> Self {
        AppError::Tauri(value)
    }
}

impl From<tauri_plugin_cli::Error> for AppError {
    fn from(value: tauri_plugin_cli::Error) -> Self {
        AppError::TauriCli(value)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::Tauri(err) => Some(err),
            AppError::TauriCli(err) => Some(err),
        }
    }
}

pub type Result<T, E = AppError> = core::result::Result<T, E>;
