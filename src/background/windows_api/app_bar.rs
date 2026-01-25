use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::state::{FancyToolbarSide, SeelenWegSide};
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    UI::Shell::{
        SHAppBarMessage, ABE_BOTTOM, ABE_LEFT, ABE_RIGHT, ABE_TOP, ABM_NEW, ABM_REMOVE, ABM_SETPOS,
        ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA,
    },
};

use crate::trace_lock;

lazy_static! {
    pub static ref RegisteredBars: Mutex<Vec<isize>> = Mutex::new(Vec::new());
}

#[allow(dead_code)]
pub enum AppBarDataEdge {
    Left = ABE_LEFT as isize,
    Top = ABE_TOP as isize,
    Right = ABE_RIGHT as isize,
    Bottom = ABE_BOTTOM as isize,
}

impl From<SeelenWegSide> for AppBarDataEdge {
    fn from(val: SeelenWegSide) -> Self {
        match val {
            SeelenWegSide::Left => AppBarDataEdge::Left,
            SeelenWegSide::Top => AppBarDataEdge::Top,
            SeelenWegSide::Right => AppBarDataEdge::Right,
            SeelenWegSide::Bottom => AppBarDataEdge::Bottom,
        }
    }
}

impl From<FancyToolbarSide> for AppBarDataEdge {
    fn from(val: FancyToolbarSide) -> Self {
        match val {
            FancyToolbarSide::Top => AppBarDataEdge::Top,
            FancyToolbarSide::Bottom => AppBarDataEdge::Bottom,
        }
    }
}

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

    pub fn set_edge(&mut self, edge: AppBarDataEdge) {
        self.0.uEdge = edge as u32;
    }

    pub fn set_rect(&mut self, rect: RECT) {
        self.0.rc = rect;
    }

    pub fn register_as_new_bar(&mut self) {
        let mut data = self.0;
        let mut registered = trace_lock!(RegisteredBars);
        let addr = data.hWnd.0 as isize;
        if !registered.contains(&addr) {
            registered.push(addr);
            unsafe { SHAppBarMessage(ABM_NEW, &mut data) };
        }
        unsafe { SHAppBarMessage(ABM_SETPOS, &mut data) };
    }

    pub fn unregister_bar(&mut self) {
        let mut data = self.0;
        unsafe { SHAppBarMessage(ABM_REMOVE, &mut data) };
        trace_lock!(RegisteredBars).retain(|x| *x != data.hWnd.0 as isize);
    }
}
