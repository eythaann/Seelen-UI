//! Overlay icons (badges) that apps set on their taskbar buttons via
//! `ITaskbarList3::SetOverlayIcon`, reported by the service hooks.

use std::{
    path::PathBuf,
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

use slu_ipc::messages::Win32TaskbarButtonEvent;
use windows::Win32::UI::WindowsAndMessaging::HICON;

use crate::{
    error::Result,
    event_manager,
    utils::{
        constants::SEELEN_COMMON, icon_extractor::convert_hicon_to_rgba_image,
        lock_free::SyncHashMap,
    },
};

static WINDOW_BADGES: LazyLock<WindowBadges> = LazyLock::new(|| WindowBadges {
    icons: SyncHashMap::new(),
});

pub struct WindowBadges {
    /// Kept apart from the tracked windows, as the overlay can be set before the window
    /// is considered interactable.
    icons: SyncHashMap<isize, WindowBadge>,
}

#[derive(Debug, Clone)]
pub enum WindowBadgesEvent {
    /// `badge` is `None` when it was removed.
    Changed {
        hwnd: isize,
        badge: Option<WindowBadge>,
    },
}

event_manager!(WindowBadges, WindowBadgesEvent);

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
                Self::send(WindowBadgesEvent::Changed { hwnd, badge });
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
}

#[derive(Debug, Clone)]
pub struct WindowBadge {
    pub icon_path: PathBuf,
    /// unix timestamp (ms) of when the badge was set
    pub updated_at: i64,
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
