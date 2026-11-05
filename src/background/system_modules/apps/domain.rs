use std::path::PathBuf;

use seelen_core::{
    rect::Rect,
    system_state::{BadgeValue, FocusedApp, MonitorId, Relaunch, UserAppWindow, WindowAttention},
};

use crate::error::Result;

/// OS contract of the apps module: the windows the user can interact with (the ones the
/// taskbar shows).
pub trait AppsSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Lists the interactable windows in z-order (top-most first). They are considered known,
    /// so only later changes are reported.
    fn list_windows(&self) -> Vec<UserAppWindow>;

    /// Starts tracking the windows, reporting their changes through `on_event`.
    /// Called once, after [`Self::list_windows`]; the tracking lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(AppsSysEvent) + Send + Sync + 'static;

    fn is_focused(&self, hwnd: isize) -> bool;

    /// Badge (count/glyph) the app has published through its notifications.
    fn badge_value(&self, umid: &str) -> Option<BadgeValue>;

    fn focused_app(&self) -> FocusedApp;

    fn mouse_position(&self) -> [i32; 2];

    /// Whether the key (by name) is pressed.
    fn is_key_down(&self, key: &str) -> Result<bool>;

    /// Minimizes (or restores) every window.
    fn toggle_desktop(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum AppsSysEvent {
    Added {
        window: Box<UserAppWindow>,
    },
    Removed {
        hwnd: isize,
    },
    Changed {
        hwnd: isize,
        change: WindowChange,
    },
    /// The badges published by the apps changed.
    BadgesChanged,
}

/// What changed on a known window.
#[derive(Debug, Clone)]
pub enum WindowChange {
    /// Title and the app identity, which some apps only set after creating the window.
    Title {
        title: String,
        app_name: String,
        umid: Option<String>,
        relaunch: Option<Relaunch>,
        prevent_pinning: bool,
    },
    Placement {
        is_zoomed: bool,
        is_fullscreen: bool,
        rect: Option<Rect>,
        monitor: MonitorId,
    },
    Foreground,
    Minimized {
        minimized: bool,
    },
    /// The window is flashing, or stopped doing it.
    Attention {
        attention: WindowAttention,
    },
    BadgeIcon {
        icon_path: Option<PathBuf>,
        updated_at: i64,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAppWinEvent {
    Added(isize),
    Updated(isize),
    Removed(isize),
}
