mod flashing;

use std::{collections::HashSet, sync::Arc, time::Duration};

use seelen_core::system_state::{BadgeValue, FocusedApp, UserAppWindow};
use windows::Win32::UI::Shell::{IShellDispatch6, Shell};

use crate::{
    error::Result,
    platform::{
        hook::{HookManager, WindowEvent},
        window::{Window, WindowsWindowExt},
    },
    system_modules::apps::domain::{AppsSysEvent, AppsSystem, WindowChange},
    utils::{lock_free::TracedMutex, spawn_named_thread},
    windows_api::{
        Com,
        input::Mouse,
        window_badges::{WindowBadges, WindowBadgesEvent},
        wpn_service::WpnService,
    },
};

/// Interval of the revalidation of the known windows, for changes without a hook event.
const REVALIDATE_INTERVAL: Duration = Duration::from_secs(5);

type EventCallback = Arc<dyn Fn(AppsSysEvent) + Send + Sync>;

pub struct WindowsApps {
    /// Windows considered interactable, to tell apart the events of unknown windows.
    known: Arc<KnownWindows>,
}

impl AppsSystem for WindowsApps {
    fn create() -> Self {
        Self {
            known: Arc::new(KnownWindows::default()),
        }
    }

    fn list_windows(&self) -> Vec<UserAppWindow> {
        let mut windows = Vec::new();
        for window in Window::enumerate().unwrap_or_default() {
            if window.is_interactable_and_not_hidden() && self.known.insert(&window) {
                windows.push(window.snapshot());
            }
        }
        windows
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(AppsSysEvent) + Send + Sync + 'static,
    {
        let on_event: EventCallback = Arc::new(on_event);

        let (known, cb) = (self.known.clone(), on_event.clone());
        HookManager::subscribe(move |(event, window)| known.on_window_event(event, window, &cb));

        let (known, cb) = (self.known.clone(), on_event.clone());
        spawn_named_thread("InteractableWindowsRevalidator", move || {
            loop {
                std::thread::sleep(REVALIDATE_INTERVAL);
                // Pause when session is not interactive to reduce CPU usage
                if crate::platform::session::is_interactive() {
                    known.revalidate(&cb);
                }
            }
        });

        flashing::spawn_flash_tracking(on_event.clone());

        let cb = on_event.clone();
        WindowBadges::subscribe(move |event| match event {
            WindowBadgesEvent::Changed { hwnd, badge } => cb(AppsSysEvent::Changed {
                hwnd,
                change: WindowChange::BadgeIcon {
                    icon_path: badge.as_ref().map(|b| b.icon_path.clone()),
                    updated_at: badge.as_ref().map_or(0, |b| b.updated_at),
                },
            }),
        });

        // Badge notifications of packaged apps (`BadgeUpdateManager`), read from the
        // notifications database.
        match WpnService::instance() {
            Ok(_) => {
                WpnService::subscribe(move |_event| on_event(AppsSysEvent::BadgesChanged));
            }
            Err(err) => log::warn!("Badge notifications will not be tracked: {err:?}"),
        }
        Ok(())
    }

    fn is_focused(&self, hwnd: isize) -> bool {
        Window::from(hwnd).is_focused()
    }

    fn badge_value(&self, umid: &str) -> Option<BadgeValue> {
        WpnService::instance().ok()?.get_badge_value(umid)
    }

    fn focused_app(&self) -> FocusedApp {
        Window::get_foregrounded().focused_app_snapshot()
    }

    fn mouse_position(&self) -> [i32; 2] {
        let point = Mouse::get_cursor_pos().unwrap_or_default();
        [point.x, point.y]
    }

    fn is_key_down(&self, key: &str) -> Result<bool> {
        use win_hotkeys::{VKey, state::KeyboardState};

        let key = VKey::from_keyname(key)?;
        if key == VKey::Menu {
            return Ok(KeyboardState::async_is_key_down(VKey::Menu.to_vk_code())
                || KeyboardState::async_is_key_down(VKey::LMenu.to_vk_code())
                || KeyboardState::async_is_key_down(VKey::RMenu.to_vk_code()));
        }
        Ok(KeyboardState::async_is_key_down(key.to_vk_code()))
    }

    fn toggle_desktop(&self) -> Result<()> {
        Com::run_with_context(|| {
            let shell: IShellDispatch6 = Com::create_instance(&Shell)?;
            unsafe { shell.ToggleDesktop()? };
            Ok(())
        })
    }
}

#[derive(Default)]
struct KnownWindows {
    hwnds: TracedMutex<HashSet<isize>>,
}

impl KnownWindows {
    fn contains(&self, window: &Window) -> bool {
        self.hwnds.lock().contains(&window.address())
    }

    /// Returns `true` if it was not known. A single locked check-and-insert avoids the race
    /// between the hook dispatcher thread and the revalidator thread both reporting it.
    fn insert(&self, window: &Window) -> bool {
        self.hwnds.lock().insert(window.address())
    }

    fn remove(&self, window: &Window) -> bool {
        self.hwnds.lock().remove(&window.address())
    }

    fn add(&self, window: &Window, on_event: &EventCallback) {
        if self.insert(window) {
            log::trace!("Adding: {window}");
            on_event(AppsSysEvent::Added {
                window: Box::new(window.snapshot()),
            });
        }
    }

    fn drop_window(&self, window: &Window, on_event: &EventCallback) {
        if self.remove(window) {
            log::trace!("Removing: {window}");
            on_event(AppsSysEvent::Removed {
                hwnd: window.address(),
            });
        }
    }

    /// Drops the windows that are no longer interactable, and adds the ones that now qualify
    /// but were never tracked (e.g. style/frame state was not settled at creation time).
    fn revalidate(&self, on_event: &EventCallback) {
        let known: Vec<isize> = self.hwnds.lock().iter().copied().collect();
        for hwnd in known {
            let window = Window::from(hwnd);
            if !window.is_interactable_and_not_hidden() {
                self.drop_window(&window, on_event);
            }
        }
        for window in Window::enumerate().unwrap_or_default() {
            if window.is_interactable_and_not_hidden() {
                self.add(&window, on_event);
            }
        }
    }

    fn on_window_event(&self, event: WindowEvent, window: Window, on_event: &EventCallback) {
        let mut is_interactable = self.contains(&window);

        match event {
            WindowEvent::Created | WindowEvent::Shown => {
                if !is_interactable && window.is_interactable_and_not_hidden() {
                    self.add(&window, on_event);
                }
            }
            WindowEvent::TitleChanged
            | WindowEvent::ParentChanged
            | WindowEvent::Uncloaked
            | WindowEvent::StateChanged => {
                let was_interactable = is_interactable;
                is_interactable = window.is_interactable_and_not_hidden();
                match (was_interactable, is_interactable) {
                    (false, true) => self.add(&window, on_event),
                    (true, false) => self.drop_window(&window, on_event),
                    _ => {}
                }

                // re-check for UWP apps that on creation starts without a parent
                if event == WindowEvent::ParentChanged
                    && let Some(parent) = window.parent()
                    && parent.is_interactable_and_not_hidden()
                {
                    self.add(&parent, on_event);
                }
            }
            WindowEvent::Hidden => {
                // UWP ApplicationFrameHosts are always hidden on minimize
                if is_interactable && !window.is_frame().unwrap_or(false) {
                    self.drop_window(&window, on_event);
                }
            }
            WindowEvent::Destroyed => {
                WindowBadges::instance().remove(window.address());
                if is_interactable {
                    self.drop_window(&window, on_event);
                }
            }
            _ => {}
        }

        if is_interactable && let Some(change) = window_change(event, &window) {
            on_event(AppsSysEvent::Changed {
                hwnd: window.address(),
                change,
            });
        }
    }
}

/// What the event changed on a known window, if anything relevant.
fn window_change(event: WindowEvent, window: &Window) -> Option<WindowChange> {
    match event {
        WindowEvent::TitleChanged => {
            // Windows has no event for property-store changes, so we piggyback on the
            // name change event to re-check the umid. Some apps (e.g. Firefox PWAs) only
            // get their AppUserModelId assigned after the window is already created.
            let umid = window.app_user_model_id();
            let (relaunch, prevent_pinning) = window.relaunch_info(&umid).unzip();
            Some(WindowChange::Title {
                title: window.title(),
                app_name: window.app_display_name().unwrap_or_default(),
                umid: umid.map(|umid| umid.to_string()),
                relaunch,
                prevent_pinning: prevent_pinning.unwrap_or(false),
            })
        }
        WindowEvent::RectChanged => Some(WindowChange::Placement {
            is_zoomed: window.is_maximized(),
            is_fullscreen: window.is_fullscreen(),
            rect: window.inner_rect().ok(),
            monitor: window.monitor().stable_id().unwrap_or_default(),
        }),
        WindowEvent::Foregrounded => Some(WindowChange::Foreground),
        WindowEvent::Minimized => Some(WindowChange::Minimized { minimized: true }),
        WindowEvent::Unminimized => Some(WindowChange::Minimized { minimized: false }),
        _ => None,
    }
}
