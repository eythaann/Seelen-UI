mod windows;

pub use windows::*;

use std::sync::LazyLock;

use seelen_core::system_state::UserAppWindow;

use crate::{event_manager, utils::lock_free::SyncVec};

pub static USER_APPS_MANAGER: LazyLock<UserAppsManager> = LazyLock::new(UserAppsManager::init);

pub struct UserAppsManager {
    pub interactable_windows: SyncVec<UserAppWindow>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserAppsEvent {
    WinAdded,
    WinUpdated,
    WinRemoved,
    AppAdded,
    AppUpdated,
    AppRemoved,
}

event_manager!(UserAppsManager, UserAppsEvent);

impl UserAppsManager {
    fn init() -> Self {
        Self {
            interactable_windows: SyncVec::from(Self::init_listing_app_windows()),
        }
    }

    fn contains_win(&self, win: &isize) -> bool {
        self.interactable_windows.any(|w| &w.hwnd == win)
    }
}
