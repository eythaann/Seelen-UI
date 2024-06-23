use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    UI::Shell::{
        SHAppBarMessage, ABE_BOTTOM, ABE_LEFT, ABE_RIGHT, ABE_TOP, ABM_GETSTATE, ABM_NEW,
        ABM_SETPOS, ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA,
    },
};

#[allow(dead_code)]
pub enum AppBarDataEdge {
    Left = ABE_LEFT as isize,
    Top = ABE_TOP as isize,
    Right = ABE_RIGHT as isize,
    Bottom = ABE_BOTTOM as isize,
}

/// https://learn.microsoft.com/en-us/windows/win32/shell/abm-setstate#parameters
#[derive(Debug, Clone, Copy)]
pub enum AppBarDataState {
    Both = 0,
    AutoHide = ABS_AUTOHIDE as isize,
    AlwaysOnTop = ABS_ALWAYSONTOP as isize,
}

impl Into<LPARAM> for AppBarDataState {
    fn into(self) -> LPARAM {
        LPARAM(self as isize)
    }
}

impl From<u32> for AppBarDataState {
    fn from(state: u32) -> Self {
        match state {
            0 => AppBarDataState::Both,
            ABS_AUTOHIDE => AppBarDataState::AutoHide,
            ABS_ALWAYSONTOP => AppBarDataState::AlwaysOnTop,
            _ => unreachable!(),
        }
    }
}

pub struct AppBarData(APPBARDATA);
impl AppBarData {
    pub fn from_handle(hwnd: HWND) -> Self {
        let mut app_bar = APPBARDATA::default();
        app_bar.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
        app_bar.hWnd = hwnd;
        Self(app_bar)
    }

    pub fn state(&self) -> AppBarDataState {
        let mut data = self.0.clone();
        AppBarDataState::from(unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) as u32 })
    }

    pub fn set_state(&self, state: AppBarDataState) {
        let mut data = self.0.clone();
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
        unsafe { SHAppBarMessage(ABM_NEW, &mut data) };
        unsafe { SHAppBarMessage(ABM_SETPOS, &mut data) };
    }
}
