use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    Graphics::Gdi::{
        RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow,
        UpdateWindow,
    },
    UI::WindowsAndMessaging::{
        BeginDeferWindowPos, DeferWindowPos, EndDeferWindowPos, GetClassNameW, GetWindowRect, HDWP,
        MoveWindow, SWP_DEFERERASE, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOOWNERZORDER,
        SWP_NOREDRAW, SWP_NOSENDCHANGING, SWP_NOSIZE, SWP_NOZORDER, SendMessageW, SetWindowPos,
    },
};

use crate::{error::Result, rect::Rect};

pub fn get_window_rect(window_id: isize) -> Result<Rect> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(HWND(window_id as _), &mut rect)? };
    Ok(rect.into())
}

#[allow(dead_code)]
pub fn start_defered_positioning(amount: i32) -> Result<HDWP> {
    let hdwp = unsafe { BeginDeferWindowPos(amount)? };
    Ok(hdwp)
}

#[allow(dead_code)]
pub fn move_window(hwnd: isize, rect: &Rect, redraw: bool) -> Result<()> {
    unsafe {
        MoveWindow(
            HWND(hwnd as _),
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            redraw,
        )?;
    }
    Ok(())
}

pub fn position_window(hwnd: isize, rect: &Rect, redraw: bool, no_size: bool) -> Result<()> {
    let mut flags = SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOZORDER | SWP_NOOWNERZORDER;

    if !redraw {
        flags |= SWP_NOREDRAW | SWP_DEFERERASE /* | SWP_NOCOPYBITS */;
    }

    if no_size {
        flags |= SWP_NOSIZE;
    }

    unsafe {
        SetWindowPos(
            HWND(hwnd as _),
            None,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            flags,
        )?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn defer_window_position(
    hdwp: HDWP,
    window_id: isize,
    rect: &Rect,
    no_size: bool,
) -> Result<HDWP> {
    let mut flags =
        SWP_NOACTIVATE | SWP_NOREDRAW | SWP_NOCOPYBITS | SWP_NOOWNERZORDER | SWP_NOZORDER;

    if no_size {
        flags |= SWP_NOSIZE;
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

#[allow(dead_code)]
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

pub fn send_message(hwnd: isize, message: u32, wparam: Option<usize>, lparam: Option<isize>) {
    unsafe {
        SendMessageW(
            HWND(hwnd as _),
            message,
            wparam.map(WPARAM),
            lparam.map(LPARAM),
        );
    }
}

pub fn get_class(hwnd: isize) -> Result<String> {
    let mut text: [u16; 512] = [0; 512];
    let len = unsafe { GetClassNameW(HWND(hwnd as _), &mut text) };
    let length = usize::try_from(len).unwrap_or(0);
    Ok(String::from_utf16(&text[..length])?)
}

pub fn is_explorer(hwnd: isize) -> Result<bool> {
    let class = get_class(hwnd as _)?;
    Ok(class == "CabinetWClass" || class == "ExplorerWClass")
}
