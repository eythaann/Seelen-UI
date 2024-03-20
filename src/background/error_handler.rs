use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    Tauri(tauri::Error),
    TauriCli(tauri_plugin_cli::Error),
    Eyre(color_eyre::eyre::Error),
    Windows(windows::core::Error),
}

macro_rules! impl_from_for_enum {
    ($enum_name:ident $(, $variant:ident ($error_type:path))*) => {
        $(
            impl From<$error_type> for AppError {
                fn from(err: $error_type) -> Self {
                    AppError::$variant(err)
                }
            }
        )*
    };
}

impl_from_for_enum!(
    AppError,
    Io(std::io::Error),
    Tauri(tauri::Error),
    TauriCli(tauri_plugin_cli::Error),
    Eyre(color_eyre::eyre::Error),
    Windows(windows::core::Error)
);

pub type Result<T, E = AppError> = core::result::Result<T, E>;

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            AppError::Tauri(err) => Some(err),
            AppError::TauriCli(err) => Some(err),
            AppError::Eyre(err) => Some(err.root_cause()),
            AppError::Windows(err) => Some(err),
        }
    }
}
