use std::sync::LazyLock;

use owo_colors::OwoColorize;
use windows::Win32::UI::Shell::FOLDERID_LocalAppData;

use crate::{error::Result, windows_api::WindowsApi};

pub struct SluServiceLogger {}

fn format_now() -> String {
    static FMT: LazyLock<Vec<time::format_description::BorrowedFormatItem>> = LazyLock::new(|| {
        time::format_description::parse("[[[year]-[month]-[day]][[[hour]:[minute]:[second]]")
            .expect("valid time format")
    });

    time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc())
        .format(&FMT)
        .unwrap_or_else(|_| "?time?".to_owned())
}

impl SluServiceLogger {
    const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024; // 5MB

    /// Legacy cleanup: removes old Windows Event Log registry entries if present.
    /// No-op in the current fern-based implementation.
    pub fn uninstall_old_logging() -> Result<()> {
        Ok(())
    }

    pub fn register_panic_hook() {
        let base_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let cause = info
                .payload()
                .downcast_ref::<String>()
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    info.payload()
                        .downcast_ref::<&str>()
                        .unwrap_or(&"<cause unknown>")
                        .to_string()
                });

            let mut string_location = String::from("<location unknown>");
            if let Some(location) = info.location() {
                string_location = format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                );
            }

            log::error!("A panic occurred:\n  Cause: {cause}\n  Location: {string_location}");
            base_hook(info);
        }));
    }

    pub fn init() -> Result<()> {
        let logs_folder =
            WindowsApi::known_folder(FOLDERID_LocalAppData)?.join("com.seelen.seelen-ui/logs");
        std::fs::create_dir_all(&logs_folder)?;

        let log_path = logs_folder.join("SLU Service.log");
        if log_path.exists() {
            let metadata = std::fs::metadata(&log_path)?;
            if metadata.len() > Self::MAX_LOG_SIZE {
                let bak_path = logs_folder.join("SLU Service.log.bak");
                std::fs::rename(&log_path, &bak_path)?;
            }
        }

        let file_dispatch = fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    format_now(),
                    record.level(),
                    record.target(),
                    message
                ))
            })
            .chain(fern::log_file(log_path)?);

        let dispatch = fern::Dispatch::new()
            .level(log::LevelFilter::Trace)
            .chain(file_dispatch);

        #[cfg(debug_assertions)]
        let dispatch = dispatch.chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}] {}",
                        match record.level() {
                            log::Level::Error => "ERROR".to_string(),
                            log::Level::Warn => "WARN~".to_string(),
                            log::Level::Info => "INFO~".to_string(),
                            log::Level::Debug => "DEBUG".to_string(),
                            log::Level::Trace => "TRACE".to_string(),
                        },
                        if record.level() == log::Level::Error {
                            record
                                .file()
                                .map(|f| {
                                    format!(
                                        "{}:{}",
                                        f.replace("\\", "/"),
                                        record.line().unwrap_or_default()
                                    )
                                })
                                .unwrap_or_else(|| record.target().to_owned())
                                .bright_red()
                                .to_string()
                        } else {
                            record.target().bright_black().to_string()
                        },
                        message
                    ))
                })
                .chain(std::io::stdout()),
        );

        dispatch.apply()?;
        Self::register_panic_hook();
        Ok(())
    }
}
