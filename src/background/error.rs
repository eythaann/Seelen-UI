macro_rules! define_app_errors {
    ($(
        $variant:ident($error_type:ty);
    )*) => {
        $(
            impl From<$error_type> for AppError {
                fn from(err: $error_type) -> Self {
                    let backtrace = backtrace::Backtrace::new();
                    AppError { msg: format!("{}({:?})", stringify!($variant), err), backtrace }
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! log_error {
    ($result:expr) => {
        if let Err(err) = $result {
            log::error!("{:?}", err);
        }
    };
    ($result:expr, $context:expr) => {
        if let Err(err) = $result {
            log::error!("Context: {:?} Err: {:?}", $context, err);
        }
    };
}

pub struct AppError {
    msg: String,
    backtrace: backtrace::Backtrace,
}

define_app_errors!(
    App(String);
    Io(std::io::Error);
    Tauri(tauri::Error);
    Lib(seelen_core::SeelenLibError);
    TauriShell(tauri_plugin_shell::Error);
    Windows(windows::core::Error);
    WMI(wmi::WMIError);
    SerdeJson(serde_json::Error);
    SerdeYaml(serde_yaml::Error);
    SerdeXml(quick_xml::de::DeError);
    Utf8(std::string::FromUtf8Error);
    Utf16(std::string::FromUtf16Error);
    CrossbeamRecv(crossbeam_channel::RecvError);
    TryFromInt(std::num::TryFromIntError);
    Image(image::ImageError);
    Battery(battery::Error);
    FileNotify(notify_debouncer_full::notify::Error);
    Base64Decode(base64::DecodeError);
    WideStringNull(widestring::error::MissingNulTerminator);
    Reqwest(tauri_plugin_http::reqwest::Error);
    Updater(tauri_plugin_updater::Error);
    WinScreenshot(win_screenshot::capture::WSError);
    EvalExpr(evalexpr::EvalexprError);
    TryFromSliceError(std::array::TryFromSliceError);
    ParseIntError(std::num::ParseIntError);
    Translator(translators::Error);
    WinHotkey(win_hotkeys::error::WHKError);
    SluIpc(slu_ipc::error::Error);
    Tokio(tokio::task::JoinError);
    Positioning(positioning::error::Error);
    Time(time::error::Error);
);

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.msg.lines() {
            if !line.is_empty() {
                writeln!(f, "{line}")?;
            }
        }

        let frames = self.backtrace.frames();
        let mut index = 0;
        for frame in frames {
            for symbol in frame.symbols() {
                let name = match symbol.name() {
                    Some(name) => name.to_string(),
                    None => continue,
                };

                // skip backtrace traces
                if name.starts_with("backtrace") {
                    continue;
                }

                // 2) skip trace of other modules/libraries specially tracing of tao and tauri libs
                if !name.starts_with("seelen_ui") {
                    index += 1;
                    continue;
                }

                // 3) skip convertion of erros to AppError
                if name.starts_with("seelen_ui::error::impl") && name.ends_with("from") {
                    index += 1;
                    continue;
                }

                writeln!(f, "    {index}: {name}")?;
                if let Some(file) = symbol.filename() {
                    write!(f, "        at: \"{}", file.to_string_lossy())?;
                    if let Some(line) = symbol.lineno() {
                        write!(f, ":{line}")?;
                        if let Some(col) = symbol.colno() {
                            write!(f, ":{col}")?;
                        }
                    }
                    writeln!(f, "\"")?;
                } else {
                    writeln!(f, "    at: <unknown>")?
                }

                index += 1;
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        err.to_owned().into()
    }
}

// needed to tauri::command macro (exposed functions to frontend)
impl From<AppError> for tauri::ipc::InvokeError {
    fn from(val: AppError) -> Self {
        tauri::ipc::InvokeError::from(val.to_string())
    }
}

impl From<tauri_plugin_shell::process::Output> for AppError {
    fn from(output: tauri_plugin_shell::process::Output) -> Self {
        let msg = if !output.stderr.is_empty() {
            let (cow, _used, _has_errors) = encoding_rs::GBK.decode(&output.stderr);
            cow.to_string().to_owned()
        } else {
            let (cow, _used, _has_errors) = encoding_rs::GBK.decode(&output.stdout);
            cow.to_string().to_owned()
        };
        let backtrace = backtrace::Backtrace::new();
        AppError { msg, backtrace }
    }
}

impl<T> From<crossbeam_channel::SendError<T>> for AppError {
    fn from(_err: crossbeam_channel::SendError<T>) -> Self {
        "Crossbeam channel disconnected".into()
    }
}

pub trait WindowsResultExt {
    /// Call this when convertion a `BOOL` into a result using the win32 crate `BOOL::ok()`
    ///
    /// For some reason `BOOL` is 0 that means failure, but the error code in the `Result` is `0`
    /// and message is `succesfully completed`
    ///
    /// Warn: Be careful when using this like win32 api documentation sometimes expect this type of behaviours...
    fn filter_fake_error(self) -> core::result::Result<(), windows::core::Error>;
}

pub trait ResultLogExt {
    /// Take the result and log it if there is an error
    fn log_error(self);
}

// todo remove this trait
pub trait ErrorMap<T> {
    fn wrap_error(self) -> core::result::Result<T, AppError>;
}

impl WindowsResultExt for core::result::Result<(), windows::core::Error> {
    fn filter_fake_error(self) -> core::result::Result<(), windows::core::Error> {
        match self {
            Ok(_) => Ok(()),
            Err(error) => {
                // I really hate windows api for this types of behaviours
                if error.code().is_ok() {
                    // let app_error = AppError::from(error);
                    // log::warn!("(maybe?) fake win32 error, was skipped: {:?}", app_error);
                    Ok(())
                } else {
                    Err(error)
                }
            }
        }
    }
}

impl<T, E: Into<AppError>> ErrorMap<T> for core::result::Result<T, E> {
    #[inline(always)]
    fn wrap_error(self) -> core::result::Result<T, AppError> {
        self.map_err(Into::into)
    }
}

impl<T, E: Into<AppError>> ResultLogExt for core::result::Result<T, E> {
    #[inline(always)]
    fn log_error(self) {
        if let Err(err) = self {
            log::error!("{:?}", err.into());
        }
    }
}

pub type Result<T = ()> = core::result::Result<T, AppError>;
