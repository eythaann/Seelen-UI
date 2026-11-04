#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{
        LazyLock,
        atomic::{AtomicIsize, Ordering},
    },
};

use seelen_core::{
    rect::Rect,
    state::WorkspaceId,
    system_state::{FocusedApp, MonitorId, UserAppWindow},
};

use crate::{
    error::Result, modules::apps::application::is_interactable_window,
    utils::lock_free::TracedMutex, virtual_desktops::VdManager,
};

use super::{monitor::Monitor, process::Process};

#[cfg(windows)]
pub use self::windows::WindowsWindowExt;

static DRAGGING_WINDOW: AtomicIsize = AtomicIsize::new(0);
static LAST_DRAGGED_WINDOW: AtomicIsize = AtomicIsize::new(0);
static RECT_BEFORE_DRAG: LazyLock<TracedMutex<Option<Rect>>> =
    LazyLock::new(|| TracedMutex::new(None));

/// A native top-level or child window of any application.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Window(imp::NativeWindow);

impl Window {
    pub fn get_foregrounded() -> Self {
        Self(imp::NativeWindow::get_foregrounded())
    }

    /// All the top-level windows, in z-order (topmost first).
    pub fn enumerate() -> Result<Vec<Self>> {
        Ok(imp::NativeWindow::enumerate()?
            .into_iter()
            .map(Self)
            .collect())
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

    pub fn process(&self) -> Process {
        Process::from_id(self.process_id())
    }

    pub fn exe_path(&self) -> Result<PathBuf> {
        self.process().exe_path()
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
        self.monitor()
            .stable_id()
            .unwrap_or_else(|_| MonitorId("null".to_string()))
    }

    pub fn workspace_id(&self) -> Result<WorkspaceId> {
        let workspace_id = VdManager::instance()
            .get_workspace_of_window(self.address())
            .ok_or("This window is not binded to a seelen ui workspace")?;
        Ok(workspace_id)
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

    /// read inner called doc for more info
    pub fn is_interactable_and_not_hidden(&self) -> bool {
        is_interactable_window(self)
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
        if self.is_minimized() {
            self.set_state(WindowState::Restored)?;
        }
        Ok(())
    }

    pub fn focus(&self) -> Result<()> {
        if self.is_minimized() {
            return Ok(());
        }
        self.0.focus()
    }

    pub fn is_dragging(&self) -> bool {
        DRAGGING_WINDOW.load(Ordering::SeqCst) == self.address()
    }

    pub fn is_last_dragged(&self) -> bool {
        LAST_DRAGGED_WINDOW.load(Ordering::SeqCst) == self.address()
    }

    pub fn set_dragging(&self, dragging: bool) {
        if dragging {
            DRAGGING_WINDOW.store(self.address(), Ordering::SeqCst);
            LAST_DRAGGED_WINDOW.store(self.address(), Ordering::SeqCst);
            *RECT_BEFORE_DRAG.lock() = self.inner_rect().ok();
        } else {
            DRAGGING_WINDOW.store(0, Ordering::SeqCst);
            // the rect is not cleaned up, so it can still be used on drag end
        }
    }

    /// if dragging returns the rect of the window before dragging
    /// otherwise returns the current rect
    pub fn get_rect_before_dragging(&self) -> Result<Rect> {
        if self.is_dragging() || self.is_last_dragged() {
            let guard = RECT_BEFORE_DRAG.lock();
            if let Some(rect) = guard.as_ref() {
                return Ok(rect.clone());
            }
        }
        self.inner_rect()
    }
}

/// Restores a window from a value previously returned by [`Window::address`].
impl From<isize> for Window {
    fn from(address: isize) -> Self {
        Self(imp::NativeWindow::from(address))
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
