macro_rules! define_app_errors {
    ($(
        $variant:ident($error_type:ty);
    )*) => {
        #[derive(Debug)]
        pub enum AppError {
            $(
                $variant($error_type),
            )*
        }

        $(
            impl From<$error_type> for AppError {
                fn from(err: $error_type) -> Self {
                    AppError::$variant(err)
                }
            }
        )*
    };
}

define_app_errors!(
    Io(std::io::Error);
    Tauri(tauri::Error);
    Eyre(color_eyre::eyre::Error);
    Windows(windows::core::Error);
    SerdeJson(serde_json::Error);
    Utf8(std::string::FromUtf8Error);
    Utf16(std::string::FromUtf16Error);
);

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Eyre(err) => Some(err.root_cause()),
            AppError::Io(err) => Some(err),
            AppError::Tauri(err) => Some(err),
            AppError::Windows(err) => Some(err),
            AppError::SerdeJson(err) => Some(err),
            AppError::Utf8(err) => Some(err),
            AppError::Utf16(err) => Some(err),
        }
    }
}

pub type Result<T = (), E = AppError> = core::result::Result<T, E>;

pub fn log_if_error<T, E>(result: Result<T, E>)
where
    E: Into<AppError>
{
    if let Err(err) = result {
        let err: AppError = err.into();
        log::error!("{:?}", err);
    }
}