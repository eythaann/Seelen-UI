use tauri::image::Image;
use tauri::path::BaseDirectory;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri::Manager;
use tauri::{
    menu::{MenuBuilder, MenuEvent, MenuItemBuilder},
    tray::TrayIconBuilder,
    AppHandle,
};

use crate::app::get_app_handle;
use crate::error_handler::Result;
use crate::log_error;
use crate::utils::sleep_millis;
use crate::widgets::show_settings;

pub fn try_register_tray_icon() -> Result<()> {
    log::trace!("registering tray icon");
    let mut attempts = 0;

    // normally tray icon creation not fails but on windows startup
    // it could fail until some processes are started
    while let Err(e) = register_tray_icon() {
        if attempts >= 10 {
            return Err(e);
        }
        attempts += 1;
        sleep_millis(100);
    }

    Ok(())
}

fn register_tray_icon() -> Result<()> {
    let app = get_app_handle();
    let settings = MenuItemBuilder::with_id("settings", t!("tray.settings")).build(app)?;
    let restart = MenuItemBuilder::with_id("restart", t!("tray.restart")).build(app)?;
    let quit = MenuItemBuilder::with_id("quit", t!("tray.quit")).build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&settings)
        .separator()
        .item(&restart)
        .separator()
        .item(&quit)
        .build()?;

    TrayIconBuilder::new()
        .icon(Image::from_path(app.path().resolve(
            "static/icons/32x32.png",
            BaseDirectory::Resource,
        )?)?)
        .tooltip("Seelen UI")
        .menu(&menu)
        .on_menu_event(
            move |app: &AppHandle, event: MenuEvent| match event.id().as_ref() {
                "settings" => {
                    log_error!(show_settings());
                }
                "restart" => app.restart(),
                "quit" => app.exit(0),
                _ => (),
            },
        )
        .on_tray_icon_event(move |_, event| {
            if let TrayIconEvent::Click {
                id: _,
                position: _,
                rect: _,
                button,
                button_state,
            } = event
            {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    log_error!(show_settings());
                }
            }
        })
        .build(app)?;

    Ok(())
}
