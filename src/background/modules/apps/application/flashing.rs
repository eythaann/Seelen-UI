//! Tracks windows that request user attention via `FlashWindow`/`FlashWindowEx`.
//!
//! Windows notifies shell hook windows on every flash toggle: `HSHELL_FLASH` when the
//! taskbar button turns highlighted and `HSHELL_REDRAW` when it turns back to normal.
//! A flash limited by `uCount` ends on `HSHELL_FLASH` (the button stays highlighted until
//! the window is activated), while `FLASHW_STOP` ends it with a single `HSHELL_REDRAW`.
//! So a flash is over when no toggle arrives for a while, and the last toggle tells
//! whether it ended highlighted or not.
//!
//! `HSHELL_REDRAW` is also sent for unrelated changes (title, icon), so it is only taken
//! into account while the window is actively flashing.

use std::{
    collections::HashMap,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use seelen_core::system_state::WindowAttention;
use windows::Win32::UI::WindowsAndMessaging::{HSHELL_HIGHBIT, HSHELL_REDRAW};

use crate::{
    modules::apps::application::{USER_APPS_MANAGER, UserAppWinEvent, UserAppsManager},
    utils::spawn_named_thread,
    windows_api::{
        event_window::{WM_SHELLHOOKMESSAGE, subscribe_to_background_window},
        window::Window,
    },
};

const HSHELL_FLASH: u32 = HSHELL_REDRAW | HSHELL_HIGHBIT;

/// Silence after which a flash is considered over. Longer than the default flash
/// toggle interval (caret blink time, ~530ms).
const FLASH_END_GRACE: Duration = Duration::from_millis(1200);

enum FlashMsg {
    On(isize),
    Redraw(isize),
}

struct ActiveFlash {
    /// the last toggle was `HSHELL_FLASH`
    lit: bool,
    last_toggle: Instant,
}

impl ActiveFlash {
    fn ends_at(&self) -> Instant {
        self.last_toggle + FLASH_END_GRACE
    }
}

/// Pure state machine over the shell messages, returns the attention changes to apply.
#[derive(Default)]
struct FlashTracker {
    active: HashMap<isize, ActiveFlash>,
}

impl FlashTracker {
    fn on_flash(&mut self, hwnd: isize, now: Instant) -> Option<(isize, WindowAttention)> {
        match self.active.get_mut(&hwnd) {
            Some(flash) => {
                flash.last_toggle = now;
                flash.lit = true;
                None
            }
            None => {
                self.active.insert(
                    hwnd,
                    ActiveFlash {
                        lit: true,
                        last_toggle: now,
                    },
                );
                Some((hwnd, WindowAttention::Flashing))
            }
        }
    }

    fn on_redraw(&mut self, hwnd: isize, now: Instant) {
        // a redraw of a window that is not flashing is not related to attention
        if let Some(flash) = self.active.get_mut(&hwnd) {
            flash.last_toggle = now;
            flash.lit = false;
        }
    }

    fn next_deadline(&self) -> Option<Instant> {
        self.active.values().map(ActiveFlash::ends_at).min()
    }

    fn on_timeout(&mut self, now: Instant) -> Vec<(isize, WindowAttention)> {
        let mut changes = Vec::new();
        self.active.retain(|hwnd, flash| {
            if flash.ends_at() > now {
                return true;
            }
            let attention = if flash.lit {
                WindowAttention::Highlighted
            } else {
                WindowAttention::None
            };
            changes.push((*hwnd, attention));
            false
        });
        changes
    }
}

impl UserAppsManager {
    pub(super) fn init_flash_tracking() {
        let (tx, rx) = crossbeam_channel::unbounded::<FlashMsg>();

        subscribe_to_background_window(move |msg, w_param, l_param| {
            if msg != WM_SHELLHOOKMESSAGE.load(Ordering::Relaxed) {
                return Ok(());
            }
            let event = match w_param as u32 {
                HSHELL_FLASH => FlashMsg::On(l_param),
                HSHELL_REDRAW => FlashMsg::Redraw(l_param),
                _ => return Ok(()),
            };
            tx.send(event)?;
            Ok(())
        });

        spawn_named_thread("FlashTracker", move || {
            let mut tracker = FlashTracker::default();
            loop {
                let received = match tracker.next_deadline() {
                    Some(deadline) => rx.recv_deadline(deadline),
                    None => rx.recv().map_err(Into::into),
                };

                match received {
                    Ok(FlashMsg::On(hwnd)) => {
                        if let Some((hwnd, attention)) = tracker.on_flash(hwnd, Instant::now()) {
                            Self::set_attention(hwnd, attention);
                        }
                    }
                    Ok(FlashMsg::Redraw(hwnd)) => tracker.on_redraw(hwnd, Instant::now()),
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        for (hwnd, attention) in tracker.on_timeout(Instant::now()) {
                            Self::set_attention(hwnd, attention);
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                }
            }
        });
    }

    fn set_attention(hwnd: isize, attention: WindowAttention) {
        let mut changed = false;
        // The foreground event clears the attention while holding this same lock, so the
        // checks below can not race with the window being activated.
        USER_APPS_MANAGER.interactable_windows.for_each(|w| {
            if w.hwnd != hwnd || w.attention == attention {
                return;
            }
            let allowed = match attention {
                // the active window is already in front of the user
                WindowAttention::Flashing => !Window::from(hwnd).is_focused(),
                // only a flash still in progress can end highlighted, not an activated window
                WindowAttention::Highlighted => w.attention == WindowAttention::Flashing,
                WindowAttention::None => true,
            };
            if allowed {
                w.attention = attention;
                changed = true;
            }
        });
        if changed {
            Self::send(UserAppWinEvent::Updated(hwnd));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HWND: isize = 0x1234;
    const TOGGLE: Duration = Duration::from_millis(530);

    /// feeds a flash of `toggles` messages starting with `HSHELL_FLASH`, returns the time of the last one
    fn flash(tracker: &mut FlashTracker, start: Instant, toggles: u32) -> Instant {
        let mut now = start;
        for i in 0..toggles {
            now = start + TOGGLE * i;
            if i % 2 == 0 {
                tracker.on_flash(HWND, now);
            } else {
                tracker.on_redraw(HWND, now);
            }
        }
        now
    }

    #[test]
    fn first_flash_starts_flashing() {
        let mut tracker = FlashTracker::default();
        let now = Instant::now();
        assert_eq!(
            tracker.on_flash(HWND, now),
            Some((HWND, WindowAttention::Flashing))
        );
        // following toggles don't produce new changes
        tracker.on_redraw(HWND, now + TOGGLE);
        assert_eq!(tracker.on_flash(HWND, now + TOGGLE * 2), None);
    }

    #[test]
    fn counted_flash_ends_highlighted() {
        let mut tracker = FlashTracker::default();
        // FLASHW_ALL with uCount = 3: on, off, on, off, on, off, on
        let last = flash(&mut tracker, Instant::now(), 7);
        assert!(tracker.on_timeout(last + TOGGLE).is_empty());
        let deadline = tracker.next_deadline().unwrap();
        assert_eq!(
            tracker.on_timeout(deadline),
            vec![(HWND, WindowAttention::Highlighted)]
        );
        assert!(tracker.next_deadline().is_none());
    }

    #[test]
    fn stopped_flash_ends_without_attention() {
        let mut tracker = FlashTracker::default();
        let last = flash(&mut tracker, Instant::now(), 4);
        // FLASHW_STOP sends a single redraw
        tracker.on_redraw(HWND, last + Duration::from_millis(100));
        let deadline = tracker.next_deadline().unwrap();
        assert_eq!(
            tracker.on_timeout(deadline),
            vec![(HWND, WindowAttention::None)]
        );
    }

    #[test]
    fn redraw_of_non_flashing_window_is_ignored() {
        let mut tracker = FlashTracker::default();
        let last = flash(&mut tracker, Instant::now(), 3);
        let deadline = tracker.next_deadline().unwrap();
        assert_eq!(
            tracker.on_timeout(deadline),
            vec![(HWND, WindowAttention::Highlighted)]
        );
        // e.g. a title change after the flash ended must not clear the highlight
        tracker.on_redraw(HWND, last + Duration::from_secs(5));
        assert!(tracker.next_deadline().is_none());
        assert!(
            tracker
                .on_timeout(last + Duration::from_secs(10))
                .is_empty()
        );
    }
}
