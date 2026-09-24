//! Tracks windows that request user attention via `FlashWindow`/`FlashWindowEx`.
//!
//! Windows notifies shell hook windows on every flash toggle: `HSHELL_FLASH` when the
//! taskbar button turns highlighted and `HSHELL_REDRAW` when it turns back to normal.
//! A flash limited by `uCount` ends on `HSHELL_FLASH` (the button stays highlighted until
//! the window is activated), while `FLASHW_STOP` ends it with a single `HSHELL_REDRAW`.
//!
//! `HSHELL_REDRAW` is also sent for unrelated changes and on every "off" toggle, so a
//! window is only considered done flashing when no new `HSHELL_FLASH` arrives within
//! [`FLASH_STOP_GRACE`] after a `HSHELL_REDRAW`. Activating the window always clears it.

use std::{
    collections::HashMap,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use windows::Win32::UI::WindowsAndMessaging::{HSHELL_HIGHBIT, HSHELL_REDRAW};

use crate::{
    modules::apps::application::{USER_APPS_MANAGER, UserAppWinEvent, UserAppsManager},
    utils::spawn_named_thread,
    windows_api::{
        WindowsApi,
        event_window::{WM_SHELLHOOKMESSAGE, subscribe_to_background_window},
    },
};

const HSHELL_FLASH: u32 = HSHELL_REDRAW | HSHELL_HIGHBIT;

/// Longer than the flash toggle interval (caret blink time, ~530ms by default).
const FLASH_STOP_GRACE: Duration = Duration::from_millis(1200);

enum FlashMsg {
    On(isize),
    Redraw(isize),
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
            // windows that were redrawn while flashing, with the time to consider them stopped
            let mut pending_stop: HashMap<isize, Instant> = HashMap::new();

            loop {
                let received = match pending_stop.values().min() {
                    Some(deadline) => rx.recv_deadline(*deadline),
                    None => rx.recv().map_err(Into::into),
                };

                match received {
                    Ok(FlashMsg::On(hwnd)) => {
                        pending_stop.remove(&hwnd);
                        // the active window is already in front of the user
                        if WindowsApi::get_foreground_window().0 as isize != hwnd {
                            Self::set_flashing(hwnd, true);
                        }
                    }
                    Ok(FlashMsg::Redraw(hwnd)) => {
                        if Self::is_flashing(hwnd) {
                            pending_stop.insert(hwnd, Instant::now() + FLASH_STOP_GRACE);
                        }
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        let now = Instant::now();
                        pending_stop.retain(|hwnd, deadline| {
                            if *deadline > now {
                                return true;
                            }
                            Self::set_flashing(*hwnd, false);
                            false
                        });
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                }
            }
        });
    }

    fn is_flashing(hwnd: isize) -> bool {
        USER_APPS_MANAGER
            .interactable_windows
            .any(|w| w.hwnd == hwnd && w.is_flashing)
    }

    fn set_flashing(hwnd: isize, flashing: bool) {
        let mut changed = false;
        USER_APPS_MANAGER.interactable_windows.for_each(|w| {
            if w.hwnd == hwnd && w.is_flashing != flashing {
                w.is_flashing = flashing;
                changed = true;
            }
        });
        if changed {
            Self::send(UserAppWinEvent::Updated(hwnd));
        }
    }
}
