use tauri::{Builder, Wry};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{Target, TargetKind};

pub fn register_plugins(app_builder: Builder<Wry>) -> Builder<Wry> {
    let log_plugin_builder = tauri_plugin_log::Builder::new()
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir { file_name: None }),
            Target::new(TargetKind::Webview),
        ])
        .level_for("tao", log::LevelFilter::Off)
        .level_for("os_info", log::LevelFilter::Off)
        .level_for("notify", log::LevelFilter::Off)
        .level_for("notify_debouncer_full", log::LevelFilter::Off)
        .level_for("discord_presence", log::LevelFilter::Off);

    let log_plugin = {
        #[cfg(not(dev))]
        {
            log_plugin_builder.build()
        }
        #[cfg(dev)]
        {
            use owo_colors::OwoColorize;
            log_plugin_builder
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "[{}][{}] {}",
                        match record.level() {
                            log::Level::Trace => "TRACE".bright_black().to_string(),
                            log::Level::Info => "INFO~".bright_blue().to_string(),
                            log::Level::Warn => "WARN~".yellow().to_string(),
                            log::Level::Error => "ERROR".red().to_string(),
                            log::Level::Debug => "DEBUG".bright_green().to_string(),
                        },
                        if record.level() == log::Level::Error {
                            record
                                .file()
                                .map(|file| {
                                    format!(
                                        "{}:{}",
                                        file.replace("\\", "/"),
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
                .build()
        }
    };

    app_builder
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--silent"]),
        ))
        .plugin(log_plugin)
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_http::init())
}
