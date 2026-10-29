#[cfg(windows)]
mod windows;

use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    path::PathBuf,
};

use seelen_core::{
    rect::Rect,
    state::{AppConfig, WorkspaceId},
    system_state::{FocusedApp, MonitorId, UserAppWindow},
};

use crate::error::Result;

use super::monitor::Monitor;

#[cfg(windows)]
pub use windows::WindowsWindowExt;

#[cfg(windows)]
type Native = windows::WindowsWindow;

/// A native top-level or child window of any application.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Window(Native);

impl Window {
    pub fn get_foregrounded() -> Self {
        Self(Native::get_foregrounded())
    }

    /// All the top-level windows, in z-order (topmost first).
    pub fn enumerate() -> Result<Vec<Self>> {
        Ok(Native::enumerate()?.into_iter().map(Self).collect())
    }

    /// Native handle as a plain number, useful to send it over IPC.
    #[inline]
    pub fn address(&self) -> isize {
        self.0.address()
    }

    pub fn is_electron(&self) -> bool {
        self.0.is_electron()
    }

    pub fn title(&self) -> String {
        self.0.title()
    }

    pub fn class(&self) -> String {
        self.0.class()
    }

    pub fn process_id(&self) -> u32 {
        self.0.process_id()
    }

    pub fn exe_path(&self) -> Result<PathBuf> {
        self.0.exe_path()
    }

    pub fn app_display_name(&self) -> Result<String> {
        self.0.app_display_name()
    }

    pub fn outer_rect(&self) -> Result<Rect> {
        self.0.outer_rect()
    }

    pub fn inner_rect(&self) -> Result<Rect> {
        self.0.inner_rect()
    }

    pub fn parent(&self) -> Option<Self> {
        self.0.parent().map(Self)
    }

    pub fn owner(&self) -> Option<Self> {
        self.0.owner().map(Self)
    }

    pub fn children(&self) -> Result<Vec<Self>> {
        Ok(self.0.children()?.into_iter().map(Self).collect())
    }

    pub fn monitor(&self) -> Monitor {
        self.0.monitor()
    }

    pub fn monitor_id(&self) -> MonitorId {
        self.0.monitor_id()
    }

    pub fn slu_category(&self) -> Option<String> {
        self.0.slu_category()
    }

    pub fn workspace_id(&self) -> Result<WorkspaceId> {
        self.0.workspace_id()
    }

    pub fn get_app_config(&self) -> Result<Option<AppConfig>> {
        self.0.get_app_config()
    }

    pub fn snapshot(&self) -> UserAppWindow {
        self.0.snapshot()
    }

    pub fn focused_app_snapshot(&self) -> FocusedApp {
        self.0.focused_app_snapshot()
    }

    /// false if the window was already destroyed.
    pub fn is_window(&self) -> bool {
        self.0.is_window()
    }

    pub fn is_visible(&self) -> bool {
        self.0.is_visible()
    }

    pub fn is_minimized(&self) -> bool {
        self.0.is_minimized()
    }

    pub fn is_maximized(&self) -> bool {
        self.0.is_maximized()
    }

    /// Hidden by the compositor while still being visible for the window system
    /// (e.g. windows of other virtual desktops or suspended apps).
    pub fn is_cloaked(&self) -> bool {
        self.0.is_cloaked()
    }

    pub fn is_focused(&self) -> bool {
        self.0.is_focused()
    }

    /// Ignores the desktop and Seelen UI widgets.
    pub fn is_fullscreen(&self) -> bool {
        self.0.is_fullscreen()
    }

    /// Windows that are part of the desktop shell (the wallpaper and desktop icons surface).
    pub fn is_desktop(&self) -> bool {
        self.0.is_desktop()
    }

    pub fn is_interactable_and_not_hidden(&self) -> bool {
        self.0.is_interactable_and_not_hidden()
    }

    pub fn set_state(&self, state: WindowState) -> Result<()> {
        self.0.set_state(state)
    }

    /// Like [`Window::set_state`] but without waiting for the window to process it.
    pub fn set_state_async(&self, state: WindowState) -> Result<()> {
        self.0.set_state_async(state)
    }

    /// Moves and resizes the window without changing its z-order nor activating it.
    pub fn set_rect(&self, rect: &Rect) -> Result<()> {
        self.0.set_rect(rect)
    }

    pub fn unminimize(&self) -> Result<()> {
        self.0.unminimize()
    }

    pub fn focus(&self) -> Result<()> {
        self.0.focus()
    }

    pub fn is_dragging(&self) -> bool {
        self.0.is_dragging()
    }

    pub fn is_last_dragged(&self) -> bool {
        self.0.is_last_dragged()
    }

    pub fn set_dragging(&self, dragging: bool) {
        self.0.set_dragging(dragging)
    }

    /// if dragging returns the rect of the window before dragging
    /// otherwise returns the current rect
    pub fn get_rect_before_dragging(&self) -> Result<Rect> {
        self.0.get_rect_before_dragging()
    }
}

/// Restores a window from a value previously returned by [`Window::address`].
impl From<isize> for Window {
    fn from(address: isize) -> Self {
        Self(Native::from(address))
    }
}

impl Hash for Window {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address().hash(state);
    }
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// Activates and shows the window.
    Shown,
    /// Shows the window without activating it.
    ShownNoActivate,
    Hidden,
    /// Activates and shows the window in its normal (not minimized nor maximized) state.
    Normal,
    Minimized,
    /// Minimizes even if the thread owning the window is not responding.
    ForceMinimized,
    Maximized,
    /// Back to the previous size and position (unminimize/unmaximize).
    Restored,
}

/// Contract to be implemented by each OS for its native window handle.
trait NativeWindow: Sized + Copy + Eq + Send + Sync + Debug + Display + From<isize> {
    fn get_foregrounded() -> Self;
    fn enumerate() -> Result<Vec<Self>>;

    fn address(&self) -> isize;
    fn is_electron(&self) -> bool;
    fn title(&self) -> String;
    fn class(&self) -> String;
    fn process_id(&self) -> u32;
    fn exe_path(&self) -> Result<PathBuf>;
    fn app_display_name(&self) -> Result<String>;
    fn outer_rect(&self) -> Result<Rect>;
    fn inner_rect(&self) -> Result<Rect>;
    fn parent(&self) -> Option<Self>;
    fn owner(&self) -> Option<Self>;
    fn children(&self) -> Result<Vec<Self>>;
    fn monitor(&self) -> Monitor;
    fn monitor_id(&self) -> MonitorId;
    fn slu_category(&self) -> Option<String>;
    fn workspace_id(&self) -> Result<WorkspaceId>;
    fn get_app_config(&self) -> Result<Option<AppConfig>>;
    fn snapshot(&self) -> UserAppWindow;
    fn focused_app_snapshot(&self) -> FocusedApp;

    fn is_window(&self) -> bool;
    fn is_visible(&self) -> bool;
    fn is_minimized(&self) -> bool;
    fn is_maximized(&self) -> bool;
    fn is_cloaked(&self) -> bool;
    fn is_focused(&self) -> bool;
    fn is_fullscreen(&self) -> bool;
    fn is_desktop(&self) -> bool;
    fn is_interactable_and_not_hidden(&self) -> bool;

    fn set_state(&self, state: WindowState) -> Result<()>;
    fn set_state_async(&self, state: WindowState) -> Result<()>;
    fn set_rect(&self, rect: &Rect) -> Result<()>;
    fn unminimize(&self) -> Result<()>;
    fn focus(&self) -> Result<()>;

    fn is_dragging(&self) -> bool;
    fn is_last_dragged(&self) -> bool;
    fn set_dragging(&self, dragging: bool);
    fn get_rect_before_dragging(&self) -> Result<Rect>;
}
