#![windows_subsystem = "windows"]
use std::process::Command;

use itertools::Itertools;

fn start_seelen_ui() -> bool {
    let current_exe = std::env::current_exe().unwrap();
    let seelen_ui_path = current_exe.with_file_name("seelen-ui.exe");
    match Command::new(seelen_ui_path).arg("--silent").spawn() {
        Ok(child) => {
            println!("Seelen-UI started with PID: {}", child.id());
            true
        }
        Err(e) => {
            eprintln!("Failed to start Seelen-UI: {}", e);
            false
        }
    }
}

fn main() {
    let mut system = sysinfo::System::new();
    system.refresh_processes();

    let is_service_already_running = system
        .processes()
        .values()
        .filter(|p| {
            p.exe()
                .is_some_and(|path| path.ends_with("slu-service.exe"))
        })
        .collect_vec()
        .len()
        > 1;

    if is_service_already_running {
        return;
    }

    let mut is_seelen_ui_running = || {
        system.refresh_processes();
        system
            .processes()
            .values()
            .any(|p| p.exe().is_some_and(|path| path.ends_with("seelen-ui.exe")))
    };

    let mut attempts = 0;
    while attempts < 5 {
        if !is_seelen_ui_running() {
            attempts += 1;
            start_seelen_ui();
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
