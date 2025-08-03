use std::{
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use windows::Win32::UI::Shell::FOLDERID_LocalAppData;

use crate::{
    enviroment::was_installed_using_msix, error::Result, was_started_from_startup_action,
    windows_api::WindowsApi,
};

pub static GUI_RESTARTED_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn launch_seelen_ui() -> Result<()> {
    let app_path = if was_installed_using_msix() {
        WindowsApi::known_folder(FOLDERID_LocalAppData)?
            .join("Microsoft\\WindowsApps\\seelen-ui.exe")
    } else {
        std::env::current_exe()?.with_file_name("seelen-ui.exe")
    };

    let mut args = Vec::new();
    if was_started_from_startup_action() {
        args.push("--startup".to_string());
    }

    let lnk_file = WindowsApi::create_temp_shortcut(&app_path, &args.join(" "))?;
    // start it using explorer to spawn it as unelevated
    Command::new("C:\\Windows\\explorer.exe")
        .arg(&lnk_file)
        .status()?;
    std::fs::remove_file(&lnk_file)?;
    Ok(())
}

pub fn kill_seelen_ui_processes() -> Result<()> {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    let instances: Vec<_> = sys
        .processes()
        .values()
        .filter(|p| p.exe().is_some_and(|path| path.ends_with("seelen-ui.exe")))
        .collect();
    for instance in instances {
        instance.kill();
    }
    GUI_RESTARTED_COUNTER.store(0, Ordering::SeqCst);
    Ok(())
}
