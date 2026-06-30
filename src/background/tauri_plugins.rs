use tauri::{Builder, Wry};

pub fn register_plugins(app_builder: Builder<Wry>) -> Builder<Wry> {
    // tauri_plugin_deep_link is intentionally omitted: the URI scheme (seelen-ui.uri) is
    // declared in tauri.conf.json only for the bundler to inject it into installer.nsi;
    // no runtime URL callback handling is needed.
    app_builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
}
