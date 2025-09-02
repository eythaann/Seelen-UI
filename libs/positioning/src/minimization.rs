/* use windows::Win32::{
    Foundation::{HWND, POINT},
    UI::WindowsAndMessaging::{
        AnimateWindow, GetWindowPlacement, IsIconic, SetWindowPlacement, ShowWindow, AW_ACTIVATE, AW_HIDE, AW_HOR_NEGATIVE, AW_HOR_POSITIVE, AW_SLIDE, SHOW_WINDOW_CMD, SW_FORCEMINIMIZE, SW_MINIMIZE, SW_SHOWMINNOACTIVE, WINDOWPLACEMENT, WPF_SETMINPOSITION
    },
};

use crate::error::Result;

fn show_window(hwnd: HWND, cmd: SHOW_WINDOW_CMD) -> Result<()> {
    let _ = unsafe { ShowWindow(hwnd, cmd) };
    Ok(())
}

fn get_window_placement(hwnd: HWND) -> Result<WINDOWPLACEMENT> {
    let mut placement = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    unsafe { GetWindowPlacement(hwnd, &mut placement)? };
    Ok(placement)
}

fn set_window_placement(hwnd: HWND, placement: &WINDOWPLACEMENT) -> Result<()> {
    unsafe { SetWindowPlacement(hwnd, placement)? };
    Ok(())
}

pub fn minimize_to_position(hwnd: HWND, x: i32, y: i32) -> Result<()> {
    if unsafe { IsIconic(hwnd).as_bool() } {
        return Ok(());
    }
    /* let mut placement = get_window_placement(hwnd)?;
    println!("PLACEMENT: {placement:?}");
    placement.flags = placement.flags | WPF_SETMINPOSITION;
    placement.ptMinPosition = POINT { x, y };
    println!("PLACEMENT: {placement:?}");
    set_window_placement(hwnd, &placement)?;

    let placement2 = get_window_placement(hwnd)?;
    println!("PLACEMENT: {placement2:?}"); */

    unsafe { AnimateWindow(hwnd, 1000, AW_SLIDE | AW_HOR_NEGATIVE | AW_HIDE)? };
    std::thread::sleep(std::time::Duration::from_millis(1000));
    // show_window(hwnd, SW_MINIMIZE)?;

    Ok(())
}
 */
