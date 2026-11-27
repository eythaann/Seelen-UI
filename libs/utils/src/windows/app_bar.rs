use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    UI::Shell::{
        ABE_BOTTOM, ABE_LEFT, ABE_RIGHT, ABE_TOP, ABM_GETSTATE, ABM_NEW, ABM_REMOVE, ABM_SETPOS,
        ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA, SHAppBarMessage,
    },
};

/// Stateless wrapper over [`APPBARDATA`] / [`SHAppBarMessage`].
///
/// It doesn't track which windows were registered as app bars: callers that need to avoid
/// double registrations must keep that state themselves.
pub struct AppBarData(pub APPBARDATA);

impl AppBarData {
    pub fn from_handle(hwnd: HWND) -> Self {
        Self(APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            ..Default::default()
        })
    }

    pub fn hwnd(&self) -> HWND {
        self.0.hWnd
    }

    pub fn with_edge(mut self, edge: AppBarEdge) -> Self {
        self.0.uEdge = edge.into();
        self
    }

    pub fn with_rect(mut self, rect: RECT) -> Self {
        self.0.rc = rect;
        self
    }

    pub fn set_state(&self, state: AppBarDataState) {
        let mut data = self.0;
        data.lParam = state.into();
        unsafe { SHAppBarMessage(ABM_SETSTATE, &mut data) };
    }

    pub fn is_auto_hide(&self) -> bool {
        let mut data = self.0;
        let state = unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) };
        state as u32 & ABS_AUTOHIDE != 0
    }

    /// `ABM_NEW`: registers the window as a new app bar. Returns `false` on failure.
    pub fn register(&self) -> bool {
        let mut data = self.0;
        unsafe { SHAppBarMessage(ABM_NEW, &mut data) != 0 }
    }

    /// `ABM_SETPOS`: reserves the configured rect on the configured edge.
    pub fn set_pos(&self) {
        let mut data = self.0;
        unsafe { SHAppBarMessage(ABM_SETPOS, &mut data) };
    }

    /// `ABM_REMOVE`: unregisters the app bar, releasing its reserved space.
    pub fn remove(&self) {
        let mut data = self.0;
        unsafe { SHAppBarMessage(ABM_REMOVE, &mut data) };
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

/// Screen edge an app bar is docked to (`ABE_*`).
#[derive(Debug, Clone, Copy)]
pub enum AppBarEdge {
    Left,
    Top,
    Right,
    Bottom,
}

impl From<AppBarEdge> for u32 {
    fn from(edge: AppBarEdge) -> Self {
        match edge {
            AppBarEdge::Left => ABE_LEFT,
            AppBarEdge::Top => ABE_TOP,
            AppBarEdge::Right => ABE_RIGHT,
            AppBarEdge::Bottom => ABE_BOTTOM,
        }
    }
}
