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
    Seelen(String);
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
    FileNotify(notify::Error);
    Base64Decode(base64::DecodeError);
    WideStringNull(widestring::error::MissingNulTerminator);
    Reqwest(tauri_plugin_http::reqwest::Error);
);

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::Seelen(err.to_owned())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// needed to tauri::command macro (exposed functions to frontend)
impl From<AppError> for tauri::ipc::InvokeError {
    fn from(val: AppError) -> Self {
        tauri::ipc::InvokeError::from(val.to_string())
    }
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        format!("{}", err)
    }
}

impl From<tauri_plugin_shell::process::Output> for AppError {
    fn from(output: tauri_plugin_shell::process::Output) -> Self {
        if !output.stderr.is_empty() {
            String::from_utf8(output.stderr)
                .unwrap_or_default()
                .trim()
                .to_owned()
                .into()
        } else {
            String::from_utf8(output.stdout)
                .unwrap_or_default()
                .trim()
                .to_owned()
                .into()
        }
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

#[macro_export]
macro_rules! log_error {
    ($($result:expr),*) => {
        $(
            if let Err(err) = $result {
                log::error!("{:?}", err);
            }
        )*
    };
}
