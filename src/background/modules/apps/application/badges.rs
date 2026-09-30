use std::{path::PathBuf, sync::LazyLock};

use slu_ipc::messages::Win32TaskbarButtonEvent;
use windows::Win32::UI::WindowsAndMessaging::HICON;

use crate::{
    error::Result,
    modules::{
        apps::application::{
            USER_APPS_MANAGER, UserAppWinEvent, UserAppsManager, windows::now_millis,
        },
        notifications::wpn_service::WpnService,
    },
    utils::{
        constants::SEELEN_COMMON, icon_extractor::convert_hicon_to_rgba_image,
        lock_free::SyncHashMap,
    },
};

static WINDOW_BADGES: LazyLock<WindowBadges> = LazyLock::new(|| WindowBadges {
    icons: SyncHashMap::new(),
});

#[derive(Debug, Clone)]
pub struct WindowBadge {
    pub icon_path: PathBuf,
    /// unix timestamp (ms) of when the badge was set
    pub updated_at: i64,
}

pub struct WindowBadges {
    /// Kept apart from the tracked windows, as the overlay can be set before the window
    /// is considered interactable.
    icons: SyncHashMap<isize, WindowBadge>,
}

impl WindowBadges {
    pub fn instance() -> &'static Self {
        &WINDOW_BADGES
    }

    pub fn process_event(&self, event: Win32TaskbarButtonEvent) {
        match event {
            Win32TaskbarButtonEvent::OverlayIconChanged { hwnd, icon_handle } => {
                let badge = match icon_handle {
                    Some(handle) => match Self::save_icon(hwnd, handle) {
                        Ok(icon_path) => Some(WindowBadge {
                            icon_path,
                            updated_at: now_millis(),
                        }),
                        Err(err) => {
                            log::warn!("Failed to read badge icon of {hwnd:#x}: {err}");
                            return;
                        }
                    },
                    None => {
                        self.remove(hwnd);
                        None
                    }
                };

                if let Some(badge) = &badge {
                    self.icons.upsert(hwnd, badge.clone());
                }

                Self::apply_to_cached_window_state(hwnd, badge);
            }
            // not used yet
            Win32TaskbarButtonEvent::OverlayDescriptionChanged { .. } => {}
        }
    }

    pub fn get(&self, hwnd: isize) -> Option<WindowBadge> {
        self.icons.get(&hwnd, |badge| badge.clone())
    }

    /// Removes the badge of the window, and its file from disk.
    pub fn remove(&self, hwnd: isize) {
        if let Some(badge) = self.icons.remove(&hwnd) {
            let _ = std::fs::remove_file(badge.icon_path);
        }
    }

    fn save_icon(hwnd: isize, icon_handle: isize) -> Result<PathBuf> {
        // the app could already have destroyed the icon, as the taskbar makes its own copy
        let image = convert_hicon_to_rgba_image(&HICON(icon_handle as _))?;
        let path = SEELEN_COMMON
            .app_temp_dir()
            .join(format!("{hwnd}_badge.png"));
        image.save(&path)?;
        Ok(path)
    }

    fn apply_to_cached_window_state(hwnd: isize, badge: Option<WindowBadge>) {
        let mut changed = false;
        USER_APPS_MANAGER.interactable_windows.for_each(|w| {
            if w.hwnd == hwnd {
                w.badge_icon_path = badge.as_ref().map(|b| b.icon_path.clone());
                w.badge_updated_at = badge.as_ref().map_or(0, |b| b.updated_at);
                changed = true;
            }
        });
        if changed {
            UserAppsManager::send(UserAppWinEvent::Updated(hwnd));
        }
    }
}

/// Badge notifications of packaged apps (`BadgeUpdateManager`), read from the notifications database.
impl UserAppsManager {
    pub(super) fn init_badge_notifications_tracking() {
        if let Err(err) = WpnService::instance() {
            log::warn!("Badge notifications will not be tracked: {err:?}");
            return;
        }
        WpnService::subscribe(|_event| Self::refresh_badge_values());
    }

    fn refresh_badge_values() {
        let Ok(service) = WpnService::instance() else {
            return;
        };

        let mut updated = Vec::new();
        USER_APPS_MANAGER.interactable_windows.for_each(|w| {
            let value = w
                .umid
                .as_deref()
                .and_then(|umid| service.get_badge_value(umid));
            if w.badge_value != value {
                w.badge_value = value;
                updated.push(w.hwnd);
            }
        });

        for hwnd in updated {
            Self::send(UserAppWinEvent::Updated(hwnd));
        }
    }
}
