use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL},
};

use crate::{
    error::Result,
    windows_api::{
        app_bar::{AppBarData, AppBarDataState},
        iterator::WindowEnumerator,
        WindowsApi,
    },
};

/// Tracks whether we actually hid the native taskbar, so we only restore it
/// when it was hidden by us (avoids unnecessary restores on settings changes/shutdown).
static NATIVE_TASKBAR_HIDDEN: AtomicBool = AtomicBool::new(false);

pub fn get_taskbars_handles() -> Result<Vec<HWND>> {
    let mut founds = Vec::new();
    WindowEnumerator::new().for_each(|hwnd| {
        let class = WindowsApi::get_class(hwnd);
        if (class == "Shell_TrayWnd" || class == "Shell_SecondaryTrayWnd")
            && WindowsApi::get_title(hwnd).is_empty()
        {
            founds.push(hwnd);
        }
    })?;
    Ok(founds)
}

pub fn hide_native_taskbar() {
    NATIVE_TASKBAR_HIDDEN.store(true, Ordering::Release);
    std::thread::spawn(|| match get_taskbars_handles() {
        Ok(handles) => {
            let mut attempts = 0;
            while attempts < 10 && NATIVE_TASKBAR_HIDDEN.load(Ordering::Acquire) {
                for hwnd in &handles {
                    AppBarData::from_handle(*hwnd).set_state(AppBarDataState::AutoHide);
                    let _ = WindowsApi::show_window_async(hwnd.0 as isize, SW_HIDE.0);
                }
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        Err(err) => log::error!("Failed to get taskbars handles: {err:?}"),
    });
}

pub fn restore_native_taskbar() -> Result<()> {
    if NATIVE_TASKBAR_HIDDEN
        .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Ok(());
    }

    for hwnd in get_taskbars_handles()? {
        AppBarData::from_handle(hwnd).set_state(AppBarDataState::AlwaysOnTop);
        WindowsApi::show_window_async(hwnd.0 as isize, SW_SHOWNORMAL.0)?;
    }
    Ok(())
}
