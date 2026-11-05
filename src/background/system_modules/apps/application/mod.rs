#[cfg(windows)]
mod windows;

use std::{
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use seelen_core::system_state::{FocusedApp, UserAppWindow, WindowAttention};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    platform::window::Window,
    utils::lock_free::SyncVec,
};

use super::domain::{AppsSysEvent, AppsSystem, UserAppWinEvent, WindowChange};

#[cfg(windows)]
type Platform = windows::WindowsApps;

pub struct UserAppsManager {
    platform: Platform,
    /// The windows the user can interact with.
    pub interactable_windows: SyncVec<UserAppWindow>,
}

event_manager!(UserAppsManager, UserAppWinEvent);

impl UserAppsManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<UserAppsManager> = LazyLock::new(|| {
            let mut m = UserAppsManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            interactable_windows: SyncVec::new(),
        }
    }

    fn init(&mut self) -> Result<()> {
        let mut windows = self.platform.list_windows();
        // The windows come in z-order (top-most first). Stamp last_foreground_at so that
        // clients sorting by that field preserve z-order on first load. Each item gets
        // now - index, keeping relative ordering without blocking any future foreground update.
        let now = now_millis();
        for (i, w) in windows.iter_mut().enumerate() {
            w.last_foreground_at = now - i as i64;
        }
        self.interactable_windows.replace(windows);
        self.platform.init(Self::on_platform_event)
    }

    fn on_platform_event(event: AppsSysEvent) {
        let manager = Self::instance();
        match event {
            AppsSysEvent::Added { window } => manager.add_window(*window),
            AppsSysEvent::Removed { hwnd } => manager.remove_window(hwnd),
            AppsSysEvent::Changed { hwnd, change } => manager.apply_change(hwnd, change),
            AppsSysEvent::BadgesChanged => manager.refresh_badge_values(),
        }
    }

    pub fn contains_win(&self, window: &Window) -> bool {
        let hwnd = window.address();
        self.interactable_windows.any(|w| w.hwnd == hwnd)
    }

    /// Single locked check-and-insert, so a window reported twice is not duplicated.
    fn add_window(&self, mut window: UserAppWindow) {
        let hwnd = window.hwnd;
        if self.platform.is_focused(hwnd) {
            window.last_foreground_at = now_millis();
        }
        let inserted = self
            .interactable_windows
            .get_or_insert_with(|w| w.hwnd == hwnd, || window);
        if inserted {
            Self::send(UserAppWinEvent::Added(hwnd));
        }
    }

    fn remove_window(&self, hwnd: isize) {
        let mut removed = false;
        self.interactable_windows.retain(|w| {
            let keep = w.hwnd != hwnd;
            removed |= !keep;
            keep
        });
        if removed {
            Self::send(UserAppWinEvent::Removed(hwnd));
        }
    }

    fn apply_change(&self, hwnd: isize, change: WindowChange) {
        let mut changed = false;
        // The foreground change clears the attention while holding this same lock, so the
        // attention checks below can not race with the window being activated.
        self.interactable_windows.for_each(|data| {
            if data.hwnd == hwnd {
                changed = self.update_window_data(data, change.clone());
            }
        });
        if changed {
            Self::send(UserAppWinEvent::Updated(hwnd));
        }
    }

    /// Returns whether the window data changed.
    fn update_window_data(&self, data: &mut UserAppWindow, change: WindowChange) -> bool {
        match change {
            WindowChange::Title {
                title,
                app_name,
                umid,
                relaunch,
                prevent_pinning,
            } => {
                data.badge_value = umid
                    .as_deref()
                    .and_then(|umid| self.platform.badge_value(umid));
                data.title = title;
                data.app_name = app_name;
                data.umid = umid;
                data.relaunch = relaunch;
                data.prevent_pinning = prevent_pinning;
            }
            WindowChange::Placement {
                is_zoomed,
                is_fullscreen,
                rect,
                monitor,
            } => {
                data.is_zoomed = is_zoomed;
                data.is_fullscreen = is_fullscreen;
                data.rect = rect;
                data.monitor = monitor;
            }
            WindowChange::Foreground => {
                data.last_foreground_at = now_millis();
                data.attention = WindowAttention::None;
            }
            WindowChange::Minimized { minimized } => {
                data.is_iconic = minimized;
            }
            WindowChange::Attention { attention } => {
                if data.attention == attention {
                    return false;
                }
                let allowed = match attention {
                    // the active window is already in front of the user
                    WindowAttention::Flashing => !self.platform.is_focused(data.hwnd),
                    // only a flash still in progress can end highlighted, not an activated window
                    WindowAttention::Highlighted => data.attention == WindowAttention::Flashing,
                    WindowAttention::None => true,
                };
                if !allowed {
                    return false;
                }
                data.attention = attention;
            }
            WindowChange::BadgeIcon {
                icon_path,
                updated_at,
            } => {
                data.badge_icon_path = icon_path;
                data.badge_updated_at = updated_at;
            }
        }
        true
    }

    fn refresh_badge_values(&self) {
        let mut updated = Vec::new();
        self.interactable_windows.for_each(|w| {
            let value = w
                .umid
                .as_deref()
                .and_then(|umid| self.platform.badge_value(umid));
            if w.badge_value != value {
                w.badge_value = value;
                updated.push(w.hwnd);
            }
        });
        for hwnd in updated {
            Self::send(UserAppWinEvent::Updated(hwnd));
        }
    }

    pub fn focused_app(&self) -> FocusedApp {
        self.platform.focused_app()
    }

    pub fn mouse_position(&self) -> [i32; 2] {
        self.platform.mouse_position()
    }

    pub fn is_key_down(&self, key: &str) -> Result<bool> {
        self.platform.is_key_down(key)
    }

    pub fn toggle_desktop(&self) -> Result<()> {
        self.platform.toggle_desktop()
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
