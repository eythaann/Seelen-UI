use color_eyre::eyre::anyhow;
use color_eyre::eyre::Result;
use tauri::{App, Error, WebviewWindow};
use tauri_plugin_cli::CliExt;

pub fn show_settings_window(app: &mut App) -> Result<WebviewWindow, Error> {
    tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("settings/index.html".into()),
    )
    .inner_size(700.0, 500.0)
    .maximizable(false)
    .minimizable(true)
    .resizable(false)
    .title("Komorebi UI - Settings")
    .visible(false)
    .decorations(false)
    .center()
    .build()
}

pub fn show_seelenpad_window(app: &mut App) -> Result<WebviewWindow, Error> {
    tauri::WebviewWindowBuilder::new(
        app,
        "seelenpad",
        tauri::WebviewUrl::App("seelenpad/index.html".into()),
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

pub fn handle_cli(app: &mut App) -> Result<()> {
    let matches = app.cli().matches().map_err(|x| anyhow!(x.to_string()))?;

    println!("{:?}", matches);

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
    } else if matches.args.len() > 0 {
        if let Some(data) = matches.args.get("help") {
            let formatted_data = data.value.to_string().trim_matches('"').replace("\\n", "\n");
            println!("{}", formatted_data);
        }

        if let Some(_) = matches.args.get("version") {
            println!("{}", app.package_info().version);
        }
    } else {
        show_settings_window(app)?;
    };

    Ok(())
}
