use tauri::{AppHandle, Builder, Wry};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_log::{
    fern::colors::{Color, ColoredLevelConfig},
    Target, TargetKind,
};

use crate::{
    cli::{handle_cli_events, SEELEN_COMMAND_LINE},
    error_handler::log_if_error,
};

pub fn register_plugins(app_builder: Builder<Wry>) -> Builder<Wry> {
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
        .plugin(tauri_plugin_single_instance::init(
            |_app: &AppHandle<Wry>, argv: Vec<String>, _cwd: String| {
                std::thread::spawn(move || {
                    let command = SEELEN_COMMAND_LINE.lock().clone();
                    log_if_error(handle_cli_events(&command.get_matches_from(argv)));
                });
            },
        ))
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .with_colors(ColoredLevelConfig {
                    error: Color::Red,
                    warn: Color::Yellow,
                    debug: Color::BrightGreen,
                    info: Color::BrightBlue,
                    trace: Color::White,
                })
                .level_for("tao", log::LevelFilter::Off)
                .build(),
        )
}
