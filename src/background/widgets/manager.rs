use std::sync::{
    LazyLock,
    atomic::{AtomicBool, Ordering},
};

use seelen_core::{
    resource::WidgetId,
    state::{WidgetLoader, WidgetStatus},
    system_state::{PowerMode, ZOrder},
};

use crate::{
    error::{Result, ResultLogExt},
    platform::{
        hook::{HookManager, WindowEvent},
        window::Window,
    },
    resources::RESOURCES,
    state::FULL_STATE,
    system_modules::{
        monitors::MonitorManager,
        power::{PowerEvent, PowerManager},
    },
    utils::lock_free::SyncHashMap,
    widgets::{WidgetWebviewLabel, loader::WidgetDeployment},
};

pub static WIDGET_MANAGER: LazyLock<WidgetManager> = LazyLock::new(WidgetManager::create);
pub static GAME_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct WidgetManager {
    /// group of widgets instances by widget resource id
    pub deployments: SyncHashMap<WidgetId, WidgetDeployment>,
}

impl WidgetManager {
    fn create() -> Self {
        let sub_id = MonitorManager::subscribe(|_event| {
            WIDGET_MANAGER.reconcile().log_error();
        });
        MonitorManager::set_event_handler_priority(&sub_id, 1);

        PowerManager::subscribe(|event| {
            if let PowerEvent::ModeChanged { mode } = event
                && FULL_STATE.load().settings.suspend_on_game_mode
            {
                match mode {
                    PowerMode::GameMode => WIDGET_MANAGER.suspend_all(),
                    _ => WIDGET_MANAGER.resume_all().log_error(),
                }
            }
        });
        // events are only emitted once the manager exists
        PowerManager::instance();

        HookManager::subscribe(|(event, _origin)| {
            if event == WindowEvent::Foregrounded {
                WIDGET_MANAGER.restore_bars_above_normal_windows();
            }
        });

        Self {
            deployments: SyncHashMap::new(),
        }
    }

    pub fn is_ready(&self, label: &WidgetWebviewLabel) -> bool {
        self.deployments
            .get(&label.widget_id, |deploy| {
                deploy.pods.any(|(key, pod)| key == label && pod.is_ready())
            })
            .unwrap_or(false)
    }

    pub fn set_status(&self, label: &WidgetWebviewLabel, status: WidgetStatus) {
        self.deployments.get(&label.widget_id, |deploy| {
            deploy.pods.get(label, |instance| {
                instance.set_status(status);
            });
        });
    }

    /// The dock and the toolbar are always-on-top windows that are never activated. Opening a
    /// WinUI 3 window (e.g. Proton VPN or PowerToys Settings) can leave them stacked below that
    /// window while they keep the always-on-top style, so every window opened afterwards also
    /// covers them until Seelen is restarted. Other always-on-top windows are not affected.
    ///
    /// When a bar ends up below a window that is not always on top, put it back on top. Bars
    /// that are only below other always-on-top windows are left alone, so this doesn't fight
    /// with them.
    fn restore_bars_above_normal_windows(&self) {
        let mut bars = Vec::new();
        for id in [WidgetId::known_weg(), WidgetId::known_toolbar()] {
            self.deployments.get(&id, |deploy| {
                deploy.pods.for_each(|(_, pod)| {
                    if let Some(hwnd) = pod.hwnd() {
                        bars.push(hwnd);
                    }
                });
            });
        }

        if bars.is_empty() {
            return;
        }

        let Ok(windows) = Window::enumerate() else {
            return;
        };

        let to_restore =
            bars_below_normal_windows(windows.iter().map(Window::address), &bars, |address| {
                let window = Window::from(address);
                window.is_visible() && !window.is_cloaked() && !window.is_always_on_top()
            });

        for address in to_restore {
            let window = Window::from(address);
            log::debug!("Restoring {window:?} above normal windows");
            window.set_z_order(ZOrder::TopMost).log_error();
        }
    }

    pub fn suspend_all(&self) {
        GAME_MODE_ACTIVE.store(true, Ordering::Release);
        self.deployments.for_each(|(_, deploy)| {
            deploy.pods.clear();
        });
    }

    pub fn resume_all(&self) -> Result<()> {
        GAME_MODE_ACTIVE.store(false, Ordering::Release);
        self.reconcile()
    }

    pub fn reconcile(&self) -> Result<()> {
        // remove deleted resources
        self.deployments
            .retain(|(key, _)| RESOURCES.widgets.contains_sync(key));

        let mut filtered = Vec::new();
        RESOURCES.widgets.iter_sync(|k, w| {
            if w.loader != WidgetLoader::Legacy {
                filtered.push((k.clone(), w.clone()));
            }
            true
        });

        let state = FULL_STATE.load();
        for (id, widget) in filtered {
            if !state.is_widget_enabled(&id) {
                self.deployments.remove(&id);
                continue;
            }

            if !self.deployments.contains_key(&id) {
                self.deployments
                    .upsert(id.clone(), WidgetDeployment::new(widget));
            }
        }

        // lazy creation of webviews to reduce startup time
        std::thread::spawn(|| {
            fn reconcile(deployment: &WidgetDeployment) {
                deployment.reconcile();
                if !deployment.definition.lazy && !GAME_MODE_ACTIVE.load(Ordering::Acquire) {
                    deployment.start_all_webviews();
                }
            }

            // More visual widgets load first
            for priority in [
                WidgetId::known_wall(),
                WidgetId::known_toolbar(),
                WidgetId::known_weg(),
            ] {
                WIDGET_MANAGER.deployments.get(&priority, |deployment| {
                    reconcile(deployment);
                });
            }

            // All other widgets
            WIDGET_MANAGER.deployments.for_each(|(_, deployment)| {
                reconcile(deployment);
            });
        });

        Ok(())
    }
}

/// Returns the `bars` that are stacked below a normal window.
///
/// `z_order` lists the windows from the top of the stack to the bottom. `is_normal_window` tells
/// whether a window is visible and not always on top; it is only called for windows above the
/// lowest bar, and not anymore once a normal window was found.
fn bars_below_normal_windows(
    z_order: impl IntoIterator<Item = isize>,
    bars: &[isize],
    is_normal_window: impl Fn(isize) -> bool,
) -> Vec<isize> {
    let mut below = Vec::new();
    let mut found = 0;
    let mut covered = false;

    for address in z_order {
        if found == bars.len() {
            break;
        }

        if bars.contains(&address) {
            found += 1;
            if covered {
                below.push(address);
            }
            continue;
        }

        if !covered && is_normal_window(address) {
            covered = true;
        }
    }

    below
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::bars_below_normal_windows;

    const DOCK: isize = 1;
    const TOOLBAR: isize = 2;
    const NORMAL: isize = 10;
    const TOPMOST: isize = 20;
    const HIDDEN_OR_CLOAKED: isize = 30;

    /// Only `NORMAL` counts as a visible window that is not always on top.
    fn is_normal(address: isize) -> bool {
        address == NORMAL
    }

    #[test]
    fn bars_above_normal_windows_are_left_alone() {
        let z_order = [DOCK, TOOLBAR, NORMAL];
        assert!(bars_below_normal_windows(z_order, &[DOCK, TOOLBAR], is_normal).is_empty());
    }

    #[test]
    fn bars_below_a_normal_window_are_restored() {
        let z_order = [NORMAL, DOCK, TOOLBAR];
        assert_eq!(
            bars_below_normal_windows(z_order, &[DOCK, TOOLBAR], is_normal),
            vec![DOCK, TOOLBAR]
        );
    }

    #[test]
    fn only_the_bars_below_a_normal_window_are_restored() {
        let z_order = [DOCK, NORMAL, TOOLBAR];
        assert_eq!(
            bars_below_normal_windows(z_order, &[DOCK, TOOLBAR], is_normal),
            vec![TOOLBAR]
        );
    }

    #[test]
    fn bars_below_always_on_top_windows_are_left_alone() {
        let z_order = [TOPMOST, DOCK, TOPMOST, TOOLBAR, NORMAL];
        assert!(bars_below_normal_windows(z_order, &[DOCK, TOOLBAR], is_normal).is_empty());
    }

    #[test]
    fn hidden_or_cloaked_windows_do_not_cover_the_bars() {
        let z_order = [HIDDEN_OR_CLOAKED, DOCK, TOOLBAR, NORMAL];
        assert!(bars_below_normal_windows(z_order, &[DOCK, TOOLBAR], is_normal).is_empty());
    }

    #[test]
    fn no_bars_means_nothing_to_restore() {
        let z_order = [NORMAL, DOCK];
        assert!(bars_below_normal_windows(z_order, &[], is_normal).is_empty());
    }

    #[test]
    fn stops_checking_windows_once_every_bar_was_found() {
        let checked = Cell::new(0);
        let z_order = [TOPMOST, DOCK, TOOLBAR, NORMAL, NORMAL, NORMAL];
        let result = bars_below_normal_windows(z_order, &[DOCK, TOOLBAR], |address| {
            checked.set(checked.get() + 1);
            is_normal(address)
        });
        assert!(result.is_empty());
        assert_eq!(
            checked.get(),
            1,
            "only the window above the bars should be checked"
        );
    }
}
