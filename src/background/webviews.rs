use tauri::{App, AppHandle, Manager, WebviewWindow};
use windows::Win32::Foundation::HWND;

use crate::{error_handler::Result, windows_api::WindowsApi, SEELEN};

pub fn focus_window(hwnd: HWND) -> Result<()> {
    // Attach komorebi thread to Window thread
    let (_, window_thread_id) = WindowsApi::window_thread_process_id(hwnd);
    let current_thread_id = WindowsApi::current_thread_id();

    // This can be allowed to fail if a window doesn't have a message queue or if a journal record
    // hook has been installed
    // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-attachthreadinput#remarks
    match WindowsApi::attach_thread_input(current_thread_id, window_thread_id, true) {
        Ok(()) => {}
        Err(error) => {
            log::error!(
                "could not attach to window thread input processing mechanism, but continuing execution of focus(): {}",
                error
            );
        }
    };

    // Raise Window to foreground
    let mut foregrounded = false;
    let mut tried_resetting_foreground_access = false;
    let mut max_attempts = 10;

    while !foregrounded && max_attempts > 0 {
        match WindowsApi::set_foreground_window(hwnd) {
            Ok(()) => {
                foregrounded = true;
            }
            Err(error) => {
                max_attempts -= 1;
                log::trace!(
                    "could not set as foreground window, but continuing execution of focus(): {}",
                    error
                );

                // If this still doesn't work then maybe try https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-locksetforegroundwindow
                if !tried_resetting_foreground_access {
                    let process_id = WindowsApi::current_process_id();
                    if WindowsApi::allow_set_foreground_window(process_id).is_ok() {
                        tried_resetting_foreground_access = true;
                    }
                }
            }
        };
    }

    // This isn't really needed when the above command works as expected
    match WindowsApi::set_focus(hwnd) {
        Ok(()) => {}
        Err(error) => {
            log::error!(
                "could not set focus, but continuing execution of focus(): {}",
                error
            );
        }
    };

    match WindowsApi::attach_thread_input(current_thread_id, window_thread_id, false) {
        Ok(()) => {}
        Err(error) => {
            log::error!(
                "could not detach from window thread input processing mechanism, but continuing execution of focus(): {}",
                error
            );
        }
    };

    Ok(())
}

pub fn show_settings_window(app: &AppHandle) -> Result<WebviewWindow> {
    log::trace!("show_settings_window");

    let window = tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("settings/index.html".into()),
    )
    .inner_size(700.0, 500.0)
    .maximizable(false)
    .minimizable(true)
    .resizable(false)
    .title("Settings")
    .visible(false)
    .decorations(false)
    .center()
    .build()?;

    Ok(window)
}

pub fn show_seelenpad_window(app: &AppHandle) -> Result<WebviewWindow> {
    log::trace!("show_seelenpad_window");

    let window = tauri::WebviewWindowBuilder::new(
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
    .shadow(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .build()?;

    focus_window(HWND(window.hwnd()?.0))?;

    Ok(window)
}

pub fn check_updates_window(app: &AppHandle) -> Result<()> {
    log::trace!("Creating update notification window");

    // check if path is in windowsapps folder
    let installation_path = app.path().resource_dir()?;
    if installation_path.starts_with(r"C:\Program Files\WindowsApps") {
        log::trace!("Skipping update notification because it is installed as MSIX");
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        app,
        "updater",
        tauri::WebviewUrl::App("update/index.html".into()),
    )
    .inner_size(500.0, 240.0)
    .maximizable(false)
    .minimizable(true)
    .resizable(false)
    .title("Update Available")
    .visible(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .center()
    .always_on_top(true)
    .build()?;

    Ok(())
}

pub fn set_windows_events(app: &mut App) -> Result<()> {
    app.listen("open-settings", |_| {
        show_settings_window(SEELEN.lock().handle()).ok();
    });

    app.listen("open-seelenpad", |_| {
        show_seelenpad_window(SEELEN.lock().handle()).ok();
    });

    Ok(())
}
