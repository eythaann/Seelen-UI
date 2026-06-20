use parking_lot::Mutex;
use seelen_core::system_state::AppBarEdge;
use std::sync::LazyLock;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    UI::Shell::{
        SHAppBarMessage, ABE_BOTTOM, ABE_LEFT, ABE_RIGHT, ABE_TOP, ABM_NEW, ABM_REMOVE, ABM_SETPOS,
        ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA,
    },
};

use crate::{error::Result, trace_lock};

static REGISTERED_BARS: LazyLock<Mutex<Vec<isize>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// https://learn.microsoft.com/en-us/windows/win32/shell/abm-setstate#parameters
#[derive(Debug, Clone, Copy)]
pub enum AppBarDataState {
    BothOff = 0,
    AutoHide = ABS_AUTOHIDE as isize,
    AlwaysOnTop = ABS_ALWAYSONTOP as isize,
    BothOn = 3,
}

impl From<AppBarDataState> for LPARAM {
    fn from(val: AppBarDataState) -> Self {
        LPARAM(val as isize)
    }
}

impl From<u32> for AppBarDataState {
    fn from(state: u32) -> Self {
        match state {
            0 => AppBarDataState::BothOff,
            ABS_AUTOHIDE => AppBarDataState::AutoHide,
            ABS_ALWAYSONTOP => AppBarDataState::AlwaysOnTop,
            3 => AppBarDataState::BothOn,
            _ => unreachable!(),
        }
    }
}

pub struct AppBarData(pub APPBARDATA);

#[allow(dead_code)]
impl AppBarData {
    pub fn from_handle(hwnd: HWND) -> Self {
        Self(APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            ..Default::default()
        })
    }

    pub fn set_state(&self, state: AppBarDataState) {
        let mut data = self.0;
        data.lParam = state.into();
        unsafe { SHAppBarMessage(ABM_SETSTATE, &mut data) };
    }

    pub fn set_edge(&mut self, edge: AppBarEdge) {
        self.0.uEdge = match edge {
            AppBarEdge::Left => ABE_LEFT,
            AppBarEdge::Top => ABE_TOP,
            AppBarEdge::Right => ABE_RIGHT,
            AppBarEdge::Bottom => ABE_BOTTOM,
        };
    }

    pub fn set_rect(&mut self, rect: RECT) {
        self.0.rc = rect;
    }

    pub fn register_as_new_bar(&mut self) -> Result<()> {
        let mut data = self.0;
        let addr = data.hWnd.0 as isize;
        let mut guard = trace_lock!(REGISTERED_BARS);

        if !guard.contains(&addr) {
            let ok = unsafe { SHAppBarMessage(ABM_NEW, &mut data) };
            if ok == 0 {
                return Err("Failed to register App Bar".into());
            }
            guard.push(addr);
        }

        unsafe { SHAppBarMessage(ABM_SETPOS, &mut data) };
        Ok(())
    }

    pub fn unregister_bar(&mut self) -> Result<()> {
        let mut data = self.0;
        let addr = data.hWnd.0 as isize;
        let mut guard = trace_lock!(REGISTERED_BARS);
        unsafe { SHAppBarMessage(ABM_REMOVE, &mut data) };
        guard.retain(|x| *x != addr);
        Ok(())
    }
}
