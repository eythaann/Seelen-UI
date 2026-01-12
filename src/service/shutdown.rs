use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::SW_SHOWNORMAL};

use crate::{
    error::Result,
    windows_api::{
        app_bar::{AppBarData, AppBarDataState},
        iterator::WindowEnumerator,
        WindowsApi,
    },
};

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

pub fn restore_native_taskbar() -> Result<()> {
    for hwnd in get_taskbars_handles()? {
        AppBarData::from_handle(hwnd).set_state(AppBarDataState::AlwaysOnTop);
        WindowsApi::show_window_async(hwnd.0 as isize, SW_SHOWNORMAL.0)?;
    }
    Ok(())
}
