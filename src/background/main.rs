// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cli::handle_cli;

mod cli;

#[tauri::command]
fn my_custom_command() {
    println!("I was invoked from JS!");
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            match app.get_cli_matches() {
                Ok(matches) => handle_cli(app, matches)?,
                Err(_) => {}
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![my_custom_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
