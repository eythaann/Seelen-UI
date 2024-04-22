pub mod virtual_desktop;

use std::time::Duration;

use tauri::{path::BaseDirectory, AppHandle, Manager, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::Foundation::RECT;

use crate::error_handler::Result;

pub fn sleep_millis(millis: u64) {
    std::thread::sleep(Duration::from_millis(millis));
}

pub fn filename_from_path(path: &str) -> String {
    path.split('\\').last().unwrap_or_default().to_string()
}

pub fn are_overlaped(rect1: &RECT, rect2: &RECT) -> bool {
    let x_overlap = !(rect1.right <= rect2.left || rect2.right <= rect1.left);
    let y_overlap = !(rect1.bottom <= rect2.top || rect2.bottom <= rect1.top);
    x_overlap && y_overlap
}

pub fn pascal_to_kebab(input: &str) -> String {
    let mut kebab_case = String::new();
    let mut prev_char_lowercase = false;
    for c in input.chars() {
        if c.is_uppercase() {
            if prev_char_lowercase {
                kebab_case.push('-');
            }
            kebab_case.push(c.to_ascii_lowercase());
            prev_char_lowercase = false;
        } else {
            kebab_case.push(c);
            prev_char_lowercase = true;
        }
    }
    kebab_case
}

pub fn kebab_to_pascal(input: &str) -> String {
    let mut pascal_case = String::new();
    let mut prev_char_dash = false;
    for c in input.chars() {
        if c == '-' {
            prev_char_dash = true;
        } else {
            if prev_char_dash || pascal_case.is_empty() {
                pascal_case.push(c.to_ascii_uppercase());
                prev_char_dash = false;
            } else {
                pascal_case.push(c);
            }
        }
    }
    pascal_case
}

pub fn is_windows_10() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if x >= &10240 && x < &22000)
}

pub fn is_windows_11() -> bool {
    matches!(os_info::get().version(), os_info::Version::Semantic(_, _, x) if x >= &22000)
}

pub fn run_ahk_file(handle: &AppHandle<Wry>, ahk_file: &str) -> Result<()> {
    log::trace!("Starting AHK: {}", ahk_file);

    let ahk_path = handle
        .path()
        .resolve("static/redis/AutoHotkey.exe", BaseDirectory::Resource)?
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_owned();

    let ahk_script_path = handle
        .path()
        .resolve(format!("static/{}", ahk_file), BaseDirectory::Resource)?
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_owned();

    handle
        .shell()
        .command(ahk_path)
        .arg(ahk_script_path)
        .spawn()?;

    Ok(())
}
