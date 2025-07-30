macro_rules! define_app_errors {
    ($(
        $variant:ident($error_type:ty);
    )*) => {
        $(
            impl From<$error_type> for ServiceError {
                fn from(err: $error_type) -> Self {
                    let backtrace = backtrace::Backtrace::new();
                    ServiceError { msg: format!("{}({:?})", stringify!($variant), err), backtrace }
                }
            }
        )*
    };
}

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

pub struct ServiceError {
    msg: String,
    backtrace: backtrace::Backtrace,
}

define_app_errors!(
    Custom(String);
    Io(std::io::Error);
    Windows(windows::core::Error);
    SerdeJson(serde_json::Error);
    Logger(log::SetLoggerError);
    WideStringNull(widestring::error::MissingNulTerminator);
    SluIpc(slu_ipc::error::Error);
);

impl std::fmt::Debug for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)?;

        let frames = self.backtrace.frames();
        if !frames.is_empty() {
            writeln!(f)?;
        }

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
                if !name.starts_with("slu_service") {
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

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<&str> for ServiceError {
    fn from(err: &str) -> Self {
        err.to_owned().into()
    }
}

impl From<std::process::Output> for ServiceError {
    fn from(output: std::process::Output) -> Self {
        if !output.stderr.is_empty() {
            let (cow, _used, _has_errors) = encoding_rs::GBK.decode(&output.stderr);
            cow.to_string().into()
        } else {
            let (cow, _used, _has_errors) = encoding_rs::GBK.decode(&output.stdout);
            cow.to_string().into()
        }
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

impl WindowsResultExt for core::result::Result<(), windows::core::Error> {
    fn filter_fake_error(self) -> core::result::Result<(), windows::core::Error> {
        match self {
            Ok(_) => Ok(()),
            Err(error) => {
                if error.code().is_ok() {
                    // log::warn!("(maybe?) fake win32 error, was skipped: {:?}", error);
                    Ok(())
                } else {
                    Err(error)
                }
            }
        }
    }
}

pub type Result<T = ()> = core::result::Result<T, ServiceError>;
