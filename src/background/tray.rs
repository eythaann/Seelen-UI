use tauri::image::Image;
use tauri::tray::ClickType;
use tauri::{
    menu::{MenuBuilder, MenuEvent, MenuItemBuilder},
    tray::TrayIconBuilder,
    App, AppHandle,
};

use crate::error_handler::Result;
use crate::windows::show_settings_window;
use crate::SEELEN;

pub fn handle_tray_icon(app: &mut App) -> Result<()> {
    let settings = MenuItemBuilder::with_id("settings", "Open Settings").build(app)?;

    let toggle_pause = MenuItemBuilder::with_id("toggle", "Pause").build(app)?;
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
        .icon(Image::from_path("./static/icons/32x32.png")?)
        .tooltip("Komorebi UI")
        .menu(&menu)
        .on_menu_event(
            move |app: &AppHandle, event: MenuEvent| match event.id().as_ref() {
                "settings" => {
                    show_settings_window(app).ok();
                }
                "pause" => {
                    println!("toggle clicked");
                }
                "restart" => app.restart(),
                "quit" => app.exit(0),
                _ => (),
            },
        )
        .on_tray_icon_event(move |_, event| match event.click_type {
            ClickType::Left | ClickType::Double => {
                show_settings_window(SEELEN.lock().handle()).ok();
            }
            ClickType::Right => {}
        })
        .build(app)?;

    Ok(())
}
