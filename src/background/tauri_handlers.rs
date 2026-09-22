use seelen_core::{
    rect::Rect, resource::*, state::by_monitor::MonitorConfiguration,
    state::by_wallpaper::WallpaperInstanceSettings, state::context_menu::*,
    state::settings::shortcuts::SystemShortcutDeclaration, state::*, system_state::*,
};

pub struct Handlers;

seelen_core::gen_tauri_wrappers!();

pub fn register_invoke_handler(
    app_builder: tauri::Builder<tauri::Wry>,
) -> tauri::Builder<tauri::Wry> {
    app_builder.invoke_handler(seelen_core::gen_tauri_invoke_handler!())
}
