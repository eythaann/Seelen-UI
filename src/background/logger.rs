use std::sync::LazyLock;

use owo_colors::OwoColorize;
use windows::Win32::UI::Shell::FOLDERID_LocalAppData;

use crate::{error::Result, windows_api::WindowsApi};

pub struct SeelenLogger {}

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

impl SeelenLogger {
    const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024; // 5MB

    pub fn init() -> Result<()> {
        let logs_folder =
            WindowsApi::known_folder(FOLDERID_LocalAppData)?.join("com.seelen.seelen-ui/logs");
        std::fs::create_dir_all(&logs_folder)?;

        let log_path = logs_folder.join("Seelen UI.log");
        if log_path.exists() {
            let metadata = std::fs::metadata(&log_path)?;
            if metadata.len() > Self::MAX_LOG_SIZE {
                let bak_path = logs_folder.join("Seelen UI.log.bak");
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
            .level_for("tao", log::LevelFilter::Off)
            .level_for("os_info", log::LevelFilter::Off)
            .level_for("notify", log::LevelFilter::Off)
            .level_for("notify_debouncer_full", log::LevelFilter::Off)
            .level_for("discord_presence", log::LevelFilter::Off)
            .chain(file_dispatch);

        #[cfg(dev)]
        let dispatch = dispatch.chain(
            fern::Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}] {}",
                        match record.level() {
                            log::Level::Error => "ERROR".red().to_string(),
                            log::Level::Warn => "WARN~".yellow().to_string(),
                            log::Level::Info => "INFO~".bright_blue().to_string(),
                            log::Level::Debug => "DEBUG".bright_green().to_string(),
                            log::Level::Trace => "TRACE".bright_black().to_string(),
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
        Ok(())
    }
}
