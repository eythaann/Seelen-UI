use tauri::{api::cli::Matches, App, Error, Window};

pub fn show_settings_window(app: &mut App) -> Result<Window, Error> {
    tauri::WindowBuilder::new(
        app,
        "settings",
        tauri::WindowUrl::App("settings/index.html".into()),
    )
    .inner_size(700.0, 500.0)
    .maximizable(false)
    .minimizable(true)
    .resizable(false)
    .title("Komorebi UI - Settings")
    .visible(false)
    .decorations(false)
    .transparent(true)
    .center()
    .build()
}

pub fn show_seelenpad_window(app: &mut App) -> Result<Window, Error> {
    tauri::WindowBuilder::new(
        app,
        "seelenpad",
        tauri::WindowUrl::App("seelenpad/index.html".into()),
    )
    .inner_size(300.0, 300.0)
    .maximizable(false)
    .minimizable(false)
    .resizable(false)
    .title("Seelenpad")
    .visible(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .build()
}

pub fn handle_cli(app: &mut App, matches: Matches) -> Result<(), Error> {
    if let Some(subcommand) = matches.subcommand {
        match subcommand.name.as_str() {
            "settings" => {
                show_settings_window(app)?;
            }
            "roulette" => {
                show_seelenpad_window(app)?;
            }
            _ => {}
        }
    } else {
        show_settings_window(app)?;
    };

    Ok(())
}
