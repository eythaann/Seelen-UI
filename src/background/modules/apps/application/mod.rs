pub mod msix;
pub mod msix_manifest;
pub mod previews;
mod windows;

pub use windows::*;

use std::sync::LazyLock;

use seelen_core::system_state::UserAppWindow;

use crate::{event_manager, utils::lock_free::SyncVec, windows_api::window::Window};

pub static USER_APPS_MANAGER: LazyLock<UserAppsManager> = LazyLock::new(UserAppsManager::init);

pub struct UserAppsManager {
    pub interactable_windows: SyncVec<UserAppWindow>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAppWinEvent {
    Added(isize),
    Updated(isize),
    Removed(isize),
}

event_manager!(UserAppsManager, UserAppWinEvent);

impl UserAppsManager {
    fn init() -> Self {
        Self {
            interactable_windows: SyncVec::from(Self::init_listing_app_windows()),
        }
    }

    pub fn instance() -> &'static Self {
        &USER_APPS_MANAGER
    }

    pub fn contains_win(&self, window: &Window) -> bool {
        let hwnd = window.address();
        self.interactable_windows.any(|w| w.hwnd == hwnd)
    }

    /// Atomically inserts `window` if it isn't already tracked. Returns `true` if it was
    /// inserted. Using a single locked check-and-insert avoids the race between the WinEvent
    /// dispatcher thread and the `InteractableWindowsRevalidator` thread both observing the
    /// window as untracked and pushing a duplicate entry.
    fn add_win(&self, window: &Window) -> bool {
        let hwnd = window.address();
        let is_focused = window.is_focused();
        self.interactable_windows.get_or_insert_with(
            |w| w.hwnd == hwnd,
            || {
                log::trace!("Adding: {window}");
                let mut serialized = window.to_serializable();
                if is_focused {
                    serialized.last_foreground_at = windows::now_millis();
                }
                serialized
            },
        )
    }

    fn remove_win(&self, window: &Window) {
        log::trace!("Removing: {window}");
        let hwnd = window.address();
        self.interactable_windows.retain(|w| w.hwnd != hwnd);
    }
}
