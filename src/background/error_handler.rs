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
    Generic(&'static str);
    GenericString(String);
    Io(std::io::Error);
    Tauri(tauri::Error);
    TauriShell(tauri_plugin_shell::Error);
    Eyre(color_eyre::eyre::Error);
    Windows(windows::core::Error);
    SerdeJson(serde_json::Error);
    SerdeYaml(serde_yaml::Error);
    Utf8(std::string::FromUtf8Error);
    Utf16(std::string::FromUtf16Error);
    CrossbeamRecv(crossbeam_channel::RecvError);
    WinVD(winvd::Error);
    TryFromInt(std::num::TryFromIntError);
    Image(image::ImageError);
    Battery(battery::Error);
);

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// needed to tauri::command macro (exposed functions to frontend)
impl Into<tauri::ipc::InvokeError> for AppError {
    fn into(self) -> tauri::ipc::InvokeError {
        tauri::ipc::InvokeError::from(self.to_string())
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        format!("{:?}", err)
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
            AppError::CrossbeamRecv(err) => Some(err),
            AppError::TauriShell(err) => Some(err),
            AppError::TryFromInt(err) => Some(err),
            _ => None,
        }
    }
}

pub type Result<T = (), E = AppError> = core::result::Result<T, E>;

pub fn log_if_error<T, E>(result: Result<T, E>)
where
    E: std::fmt::Debug,
{
    if let Err(err) = result {
        log::error!("{:?}", err);
    }
}
