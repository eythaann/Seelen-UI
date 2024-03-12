use tauri::App;
use tauri_plugin_cli::CliExt;

use crate::error_handler::Result;
use crate::windows::show_seelenpad_window;
use crate::windows::show_settings_window;

type ShouldInitApp = bool;

pub fn handle_cli(app: &mut App) -> Result<ShouldInitApp> {
    let matches = app.cli().matches()?;

    if let Some(data) = matches.args.get("verbose") {
        if data.value == true {
            println!("{:?}", matches);
        }
    }

    if let Some(subcommand) = matches.subcommand {
        match subcommand.name.as_str() {
            "settings" => {
                show_settings_window(app.handle())?;
            }
            "roulette" => {
                show_seelenpad_window(app.handle())?;
            }
            _ => {}
        }
        return Ok(true);
    }

    if let Some(data) = matches.args.get("silent") {
        if data.value != false {
            return Ok(true);
        }
    }

    if let Some(data) = matches.args.get("help") {
        let formatted_data = data
            .value
            .to_string()
            .trim_matches('"')
            .replace("\\n", "\n");
        println!("{}", formatted_data);
        return Ok(false);
    }

    if matches.args.contains_key("version") {
        println!("{}", app.package_info().version);
        return Ok(false);
    }

    show_settings_window(app.handle())?;
    Ok(true)
}
