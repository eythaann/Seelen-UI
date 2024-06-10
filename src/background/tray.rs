use tauri::image::Image;
use tauri::path::BaseDirectory;
use tauri::tray::ClickType;
use tauri::Manager;
use tauri::{
    menu::{MenuBuilder, MenuEvent, MenuItemBuilder},
    tray::TrayIconBuilder,
    App, AppHandle,
};

use crate::error_handler::{log_if_error, Result};
use crate::seelen::SEELEN;

pub fn handle_tray_icon(app: &mut App) -> Result<()> {
    log::trace!("registering tray icon");
    let settings = MenuItemBuilder::with_id("settings", "Open Settings").build(app)?;

    let toggle_pause = MenuItemBuilder::with_id("pause", "Pause/Resume").build(app)?;
    let restart = MenuItemBuilder::with_id("restart", "Reload").build(app)?;

    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&settings)
        .separator()
        .item(&toggle_pause)
        .item(&restart)
        .separator()
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .icon(Image::from_path(app.path().resolve("static/icons/32x32.png", BaseDirectory::Resource)?)?)
        .tooltip("Seelen UI")
        .menu(&menu)
        .on_menu_event(
            move |app: &AppHandle, event: MenuEvent| match event.id().as_ref() {
                "settings" => {
                    log_if_error(SEELEN.lock().show_settings());
                }
                "pause" => {
                }
                "restart" => app.restart(),
                "quit" => app.exit(0),
                _ => (),
            },
        )
        .on_tray_icon_event(move |_, event| match event.click_type {
            ClickType::Left | ClickType::Double => {
                log_if_error(SEELEN.lock().show_settings());
            }
            ClickType::Right => {}
        })
        .build(app)?;

    Ok(())
}
