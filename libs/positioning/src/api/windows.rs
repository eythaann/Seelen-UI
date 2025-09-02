use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{
        RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow,
        UpdateWindow,
    },
    UI::WindowsAndMessaging::{
        BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, GetWindowRect, HDWP,
        SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOREDRAW, SWP_NOZORDER,
    },
};

use crate::{error::Result, rect::Rect};

pub fn get_window_rect(window_id: isize) -> Result<Rect> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(HWND(window_id as _), &mut rect)? };
    Ok(rect.into())
}

pub fn start_defered_positioning(amount: i32) -> Result<HDWP> {
    let hdwp = unsafe { BeginDeferWindowPos(amount)? };
    Ok(hdwp)
}

/*
fn position_window(
    hwnd: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> windows::core::Result<HDWP> {
    unsafe {
        SetWindowPos(
            hwnd,
            None,
            x,
            y,
            width,
            height,
            SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOOWNERZORDER | SWP_NOZORDER,
        )
    }
}
 */

pub fn defer_window_position(
    hdwp: HDWP,
    window_id: isize,
    rect: &Rect,
    redraw: bool,
) -> Result<HDWP> {
    let mut flags = SWP_NOACTIVATE | SWP_NOZORDER;
    if !redraw {
        flags |= SWP_NOREDRAW | SWP_NOCOPYBITS;
    }
    let hdwp = unsafe {
        DeferWindowPos(
            hdwp,
            HWND(window_id as _),
            None,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            flags,
        )?
    };
    Ok(hdwp)
}

pub fn finish_defered_positioning(hdwp: HDWP) -> Result<()> {
    unsafe { EndDeferWindowPos(hdwp)? };
    Ok(())
}

pub fn force_redraw_window(window_id: isize) -> Result<()> {
    unsafe {
        let hwnd = HWND(window_id as _);
        RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN | RDW_FRAME | RDW_ERASE,
        )
        .ok()?;
        UpdateWindow(hwnd).ok()?;
    }
    Ok(())
}
